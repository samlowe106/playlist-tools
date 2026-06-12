use crate::models::{
    PlaylistItem, PlaylistItemListResponse, UpdateBody, UpdateSnippet, VideoListResponse,
};
use anyhow::{Context, Result};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use url::Url;

pub async fn get_oauth_token() -> anyhow::Result<String> {
    let client_id = std::env::var("YOUTUBE_CLIENT_ID").context("YOUTUBE_CLIENT_ID not set")?;
    let client_secret =
        std::env::var("YOUTUBE_CLIENT_SECRET").context("YOUTUBE_CLIENT_SECRET not set")?;

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string())?);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/youtube.force-ssl".to_string(),
        ))
        .url();

    println!("Opening browser for authentication...");
    open::that(auth_url.to_string())?;

    // Spin up a local server to catch the redirect
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (mut stream, _) = listener.accept().await?;

    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    // Parse code and state from GET /?code=...&state=...
    let redirect_url = request_line
        .split_whitespace()
        .nth(1)
        .context("Invalid redirect request")?;
    let url = Url::parse(&format!("http://localhost{}", redirect_url))?;

    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| AuthorizationCode::new(v.into_owned()))
        .context("No code in redirect")?;

    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| CsrfToken::new(v.into_owned()))
        .context("No state in redirect")?;

    // Send a response so the user sees something in the browser
    let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h2>Authenticated! You can close this tab.</h2></body></html>";
    stream.write_all(response.as_bytes()).await?;

    // Verify CSRF token
    anyhow::ensure!(state.secret() == csrf_token.secret(), "CSRF token mismatch");

    // Exchange the code for an access token
    let token = client
        .exchange_code(code)
        .request_async(async_http_client)
        .await
        .context("Failed to exchange code for token")?;

    Ok(token.access_token().secret().to_owned())
}

pub async fn fetch_playlist_title(
    client: &Client,
    playlist_id: &str,
    api_key: &str,
    oauth_token: &str,
) -> anyhow::Result<String> {
    let response = client
        .get("https://www.googleapis.com/youtube/v3/playlists")
        .bearer_auth(oauth_token)
        .query(&[("part", "snippet"), ("id", playlist_id), ("key", api_key)])
        .send()
        .await
        .context("GET playlists failed")?
        .error_for_status()
        .context("YouTube API returned an error on playlists.list")?;

    let page: crate::models::PlaylistListResponse = response
        .json()
        .await
        .context("Failed to deserialise playlists.list response")?;

    page.items
        .into_iter()
        .next()
        .map(|p| p.snippet.title)
        .context("Playlist not found")
}

pub async fn fetch_all_items(
    client: &Client,
    playlist_id: &str,
    api_key: &str,
    oauth_token: &str,
) -> Result<Vec<PlaylistItem>> {
    /* Fetches playlist */
    let mut all_items: Vec<PlaylistItem> = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let params: HashMap<&str, String> = [
            ("part", "snippet,contentDetails".into()),
            ("playlistId", playlist_id.into()),
            ("maxResults", "50".into()),
            ("key", api_key.into()),
        ]
        .into_iter()
        .chain(page_token.as_deref().map(|t| ("pageToken", t.to_string())))
        .collect();

        let response = client
            .get("https://www.googleapis.com/youtube/v3/playlistItems")
            .bearer_auth(oauth_token)
            .query(&params)
            .send()
            .await
            .context("GET playlistItems failed")?
            .error_for_status()
            .context("YouTube API returned an error status")?;

        let page: PlaylistItemListResponse = response
            .json()
            .await
            .context("Failed to deserialise playlistItems response")?;

        all_items.extend(page.items);

        match page.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(all_items)
}

pub async fn fetch_durations(
    client: &Client,
    items: &[PlaylistItem],
    api_key: &str,
    oauth_token: &str,
) -> Result<HashMap<String, u64>> {
    let mut durations: HashMap<String, u64> = HashMap::new();

    for chunk in items.chunks(50) {
        let ids: Vec<&str> = chunk
            .iter()
            .map(|i| i.content_details.video_id.as_str())
            .collect();

        let response = client
            .get("https://www.googleapis.com/youtube/v3/videos")
            .bearer_auth(oauth_token)
            .query(&[
                ("part", "contentDetails"),
                ("id", &ids.join(",")),
                ("key", api_key),
            ])
            .send()
            .await
            .context("GET videos failed")?
            .error_for_status()
            .context("YouTube API returned an error on videos.list")?;

        let page: VideoListResponse = response
            .json()
            .await
            .context("Failed to deserialise videos.list response")?;

        for video in page.items {
            let secs = parse_iso_duration(&video.content_details.duration);
            durations.insert(video.id, secs);
        }
    }

    Ok(durations)
}

pub async fn push_new_order(
    client: &Client,
    items: &[PlaylistItem],
    playlist_id: &str,
    api_key: &str,
    oauth_token: &str,
    start_position: usize,
) -> Result<()> {
    for (position, item) in items.iter().enumerate() {
        // Skip items before the start position
        if position < start_position {
            continue;
        }

        // Skip items that haven't actually moved to save quota
        if item.snippet.position == position as u32 {
            println!("  {} '{}' (SAME)", position, item.snippet.title);
            continue;
        }

        let body = UpdateBody {
            id: item.id.clone(),
            snippet: UpdateSnippet {
                playlist_id: playlist_id.to_owned(),
                position: position as u32,
                resource_id: item.snippet.resource_id.clone(),
            },
        };

        client
            .put("https://www.googleapis.com/youtube/v3/playlistItems")
            .bearer_auth(oauth_token)
            .query(&[("part", "snippet"), ("key", api_key)])
            .json(&body)
            .send()
            .await
            .context(format!("PUT playlistItems failed for item {}", item.id))?
            .error_for_status()
            .context(format!("YouTube returned an error for item {}", item.id))?;

        println!("  {} '{}' (MOVED)", position, item.snippet.title);
    }

    Ok(())
}

/*
pub fn parse_iso_duration(s: &str) -> u64 {
    let s = s.trim_start_matches("PT");
    let hours = s
        .find('H')
        .map(|i| s[..i].parse::<u64>().unwrap_or(0))
        .unwrap_or(0);
    let mins_s = s.find('H').map(|i| &s[i + 1..]).unwrap_or(s);
    let mins = mins_s
        .find('M')
        .map(|i| mins_s[..i].parse::<u64>().unwrap_or(0))
        .unwrap_or(0);
    let secs_s = mins_s.find('M').map(|i| &mins_s[i + 1..]).unwrap_or(mins_s);
    let secs = secs_s.trim_end_matches('S').parse::<u64>().unwrap_or(0);
    hours * 3600 + mins * 60 + secs
}
 */

pub fn parse_iso_duration(s: &str) -> u64 {
    let s = s.trim_start_matches("PT");
    let (s, hours) = match s.find('H') {
        Some(i) => (&s[i + 1..], s[..i].parse::<u64>().unwrap_or(0)),
        None => (s, 0),
    };
    let (s, mins) = match s.find('M') {
        Some(i) => (&s[i + 1..], s[..i].parse::<u64>().unwrap_or(0)),
        None => (s, 0),
    };
    let secs = s.trim_end_matches('S').parse::<u64>().unwrap_or(0);
    hours * 3600 + mins * 60 + secs
}

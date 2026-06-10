use crate::models::{
    PlaylistItem, PlaylistItemListResponse, UpdateBody, UpdateSnippet, VideoListResponse,
};
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::HashMap;

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
        // can params and the params.insert be combined and made immutable?
        let mut params: HashMap<&str, String> = HashMap::from([
            ("part", "snippet,contentDetails".into()),
            ("playlistId", playlist_id.into()),
            ("maxResults", "50".into()), // 50 is the API maximum
            ("key", api_key.into()),
        ]);
        if let Some(ref token) = page_token {
            params.insert("pageToken", token.clone());
        }

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
) -> Result<()> {
    /* Pushes new order to YouTube API */
    for (position, item) in items.iter().enumerate() {
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
        println!("  Moved '{}' to position {}", item.snippet.title, position);
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

fn parse_iso_duration(s: &str) -> u64 {
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

use anyhow::Context;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;

use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
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

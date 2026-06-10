# 🔀 Playlist Tools

A command line program to sort YouTube playlists by additional metrics including duration and uploader name using the YouTube Data API v3.

## Setup
### 1. Prerequisites

1. Rust installed via [rustup](https://rustup.rs/)
2. A Google account with a YouTube channel

### 2. Google Cloud Console

1. Create a new `.env` file in the project root:
    ```env
    YOUTUBE_API_KEY=your_api_key
    YOUTUBE_CLIENT_ID=your_client_id.apps.googleusercontent.com
    YOUTUBE_CLIENT_SECRET=your_client_secret
    ```
    and fill in those values

2. Go to [console.cloud.google.com](https://console.cloud.google.com) and create a new project
3. Go to APIs & Services => Library, search for "YouTube Data API v3" and enable it
4. Go to APIs & Services => OAuth consent screen, set it to Testing mode, and add your Google account as a test user
5. Go to APIs & Services => Credentials and click Create Credentials => API key. Copy that value
6. On the same page click Create Credentials => OAuth client ID, choose Desktop app, and copy the client ID and secret. Your `.env` file should now look like:

    ```env
    YOUTUBE_API_KEY=your_api_key
    YOUTUBE_CLIENT_ID=your_client_id.apps.googleusercontent.com
    YOUTUBE_CLIENT_SECRET=your_client_secret
    ```

### 3. Build and run

```bash
cargo build --release
./target/release/playlist-tools --playlist-id "https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID"
```

On first run, your browser window will open asking you to authenticate with Google. After approving, the app will fetch your playlist, sort it, and push the new order to YouTube.

### 4. Working around quota limits

The YouTube API has a daily quota of 10,000 units. Each video reorder costs 50 units, so large playlists may need to be processed over multiple days. Use `--start-position N` to resume from where you left off:
```bash
./target/release/playlist-tools --playlist-id "https://..." --start-position N
```
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoListResponse {
    pub items: Vec<Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub content_details: VideoContentDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoContentDetails {
    pub duration: String, // ISO 8601, e.g. "PT4M33S"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemListResponse {
    pub next_page_token: Option<String>,
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub id: String, // The playlistItem ID (not the video ID)
    pub snippet: Snippet,
    pub content_details: PlaylistItemContentDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub title: String,
    pub published_at: String, // RFC 3339 — used for date ordering
    pub position: u32,
    pub playlist_id: String,
    pub resource_id: ResourceId,
    pub video_owner_channel_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceId {
    pub kind: String,
    // Option because items can reference channels or playlists, not just videos
    pub video_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemContentDetails {
    pub video_id: String,
    // Option because the API omits this for private or deleted videos
    pub video_published_at: Option<String>,
}

// The body we PUT back to YouTube for each item whose position changed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBody {
    pub id: String,
    pub snippet: UpdateSnippet,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnippet {
    pub playlist_id: String,
    pub position: u32,
    pub resource_id: ResourceId,
}

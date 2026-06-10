use anyhow::Context;
use std::collections::HashMap;

const CACHE_PATH: &str = "durations_cache.json";

fn load_cache() -> HashMap<String, u64> {
    std::fs::read_to_string(CACHE_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_cache(cache: &HashMap<String, u64>) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(cache)?;
    std::fs::write(CACHE_PATH, json).context("Failed to write duration cache")
}

pub async fn fetch_durations(
    client: &reqwest::Client,
    items: &[crate::models::PlaylistItem],
    api_key: &str,
    oauth_token: &str,
) -> anyhow::Result<HashMap<String, u64>> {
    let mut cache = load_cache();

    // Only fetch IDs we don't already have
    let uncached: Vec<&str> = items
        .iter()
        .map(|i| i.content_details.video_id.as_str())
        .filter(|id| !cache.contains_key(*id))
        .collect();

    if uncached.is_empty() {
        println!("  All durations loaded from cache.");
        return Ok(cache);
    }

    println!(
        "  Fetching {} durations ({} cached)...",
        uncached.len(),
        cache.len()
    );

    cache.extend(crate::api::fetch_durations(client, items, api_key, oauth_token).await?);

    save_cache(&cache)?;
    println!("  Cache saved to {}", CACHE_PATH);

    // Return only the durations for items in this playlist
    Ok(cache)
}

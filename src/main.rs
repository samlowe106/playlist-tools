use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
mod api;
mod cache;
mod models;
mod sorting;
mod visuals;
use api::{fetch_all_items, fetch_playlist_title, get_oauth_token, push_new_order};
use reqwest::Client;
use sorting::{SortOrder, sort_items};
use url::Url;
use visuals::draw_chart;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    playlist_id: String,
    #[arg(short, long, default_value = "duration")]
    order: SortOrder,
    /// Sort in ascending order (least to greatest)
    #[arg(short, long, default_value_t = true)]
    ascending: bool,
    /// Resume pushing from this position (useful if you hit the daily API quota limit in a previous run)
    #[arg(long, default_value_t = 0)]
    start_position: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let api_key = std::env::var("YOUTUBE_API_KEY").context("YOUTUBE_API_KEY not set")?;
    let oauth_token = get_oauth_token().await?;

    let parsed = Url::parse(args.playlist_id.trim()).context("Invalid URL")?;
    let playlist_id = parsed
        .query_pairs()
        .find(|(key, _)| key == "list")
        .map(|(_, value)| value.into_owned())
        .context("No playlist ID found in URL")?;

    let client = Client::new();

    let playlist_title = fetch_playlist_title(
        &client,
        playlist_id.as_str(),
        api_key.as_str(),
        oauth_token.as_str(),
    )
    .await?;
    println!("Found playlist {playlist_title}, fetching playlist items...");

    let mut items = fetch_all_items(&client, &playlist_id, &api_key, &oauth_token).await?;
    println!("  {} items fetched.", items.len());

    let durations = if args.order == SortOrder::Duration {
        let durations = cache::fetch_durations(&client, &items, &api_key, &oauth_token).await?;
        println!("  Durations fetched.");
        durations
    } else {
        HashMap::new()
    };

    println!("Sorting ({:?})…", args.order);
    sort_items(&mut items, args.order, &durations, args.ascending);

    if args.order == SortOrder::Duration {
        draw_chart(
            format!("{} Video Durations", playlist_title).as_str(),
            &durations.into_values().collect::<Vec<u64>>(),
        )?;
    }

    println!("Pushing new order…");
    push_new_order(
        &client,
        &items,
        &playlist_id,
        &api_key,
        &oauth_token,
        args.start_position,
    )
    .await?;

    Ok(())
}

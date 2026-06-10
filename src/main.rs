use std::collections::HashMap;

mod auth;
use auth::get_oauth_token;

use anyhow::Context;
use clap::Parser;
mod api;
mod models;
mod sorting;
mod visuals;
use api::{fetch_all_items, fetch_durations, push_new_order};
use reqwest::Client;
use sorting::{SortOrder, sort_items};
use url::Url;
use visuals::draw_chart;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    playlist_id: String,
    #[arg(short, long, default_value_t = SortOrder::Duration)]
    order: SortOrder,
    #[arg(short, long, default_value_t = true)]
    ascending: bool,
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

    println!("Fetching playlist items for {playlist_id}...");
    let mut items = fetch_all_items(&client, &playlist_id, &api_key, &oauth_token).await?;
    println!("  {} items fetched.", items.len());

    let durations = if args.order == SortOrder::Duration {
        let durations = fetch_durations(&client, &items, &api_key, &oauth_token).await?;
        println!("  Durations fetched.");
        durations
    } else {
        HashMap::new()
    };

    println!("Sorting ({:?})…", args.order);
    sort_items(&mut items, args.order, &durations, args.ascending);

    if args.order == SortOrder::Duration {
        draw_chart(&durations.into_values().collect::<Vec<u64>>())?;
    }

    println!("Pushing new order…");
    push_new_order(&client, &items, &playlist_id, &api_key, &oauth_token).await?;

    Ok(())
}

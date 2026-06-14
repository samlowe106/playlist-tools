use std::collections::HashMap;

use anyhow::Context;
use clap::Parser;
mod api;
mod cache;
mod models;
mod sorting;
mod visuals;
use api::{
    fetch_all_items, fetch_playlist_title, get_oauth_token, push_new_order, remove_duplicates,
};
use reqwest::Client;
use sorting::{SortOrder, sort_items};
use url::Url;
use visuals::{GraphOption, draw_durations_chart, draw_uploaders_chart};

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
    /// Delete duplicates from the playlist
    #[arg(long, default_value_t = false)]
    remove_duplicates: bool,
    /// Graph statistics
    #[arg(long)]
    graph: Vec<GraphOption>,
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
    print!("Found playlist {playlist_title}, fetching playlist items...");

    let mut items = fetch_all_items(&client, &playlist_id, &api_key, &oauth_token).await?;
    println!("{} items fetched.", items.len());

    let durations = {
        let durations_needed =
            args.order == SortOrder::Duration || args.graph.contains(&GraphOption::Durations);

        if durations_needed {
            cache::fetch_durations(&client, &items, &api_key, &oauth_token).await?
        } else {
            HashMap::new()
        }
    };

    if args.remove_duplicates {
        let removed = remove_duplicates(&client, &items, &api_key, &oauth_token).await?;
        println!("  Removed {} duplicates.", removed);
        // Re-fetch items so positions are accurate after deletions
        items = fetch_all_items(&client, &playlist_id, &api_key, &oauth_token).await?;
    }

    println!("Sorting by ({:?})…", args.order);
    sort_items(&mut items, args.order, &durations, args.ascending);

    for graph in args.graph {
        match graph {
            GraphOption::Durations => {
                draw_durations_chart(
                    &format!("{} Video Durations", playlist_title),
                    &durations.values().copied().collect::<Vec<u64>>(),
                )?;
            }
            GraphOption::Uploaders => {
                let uploaders: Vec<String> = items
                    .iter()
                    .filter_map(|i| i.snippet.video_owner_channel_title.clone())
                    .collect();
                draw_uploaders_chart(&format!("{} Uploaders", playlist_title), &uploaders)?;
            }
        }
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

use std::collections::HashMap;

use anyhow::{Context, Result};
use dotenv;
mod api;
mod models;
mod sorting;
mod visuals;
use api::{fetch_all_items, fetch_durations, push_new_order};
use reqwest::Client;
use sorting::{SortOrder, sort_items};
use strum::IntoEnumIterator;
use url::Url;
use visuals::draw_chart;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    let api_key = std::env::var("YOUTUBE_API_KEY").context("YOUTUBE_API_KEY not set")?;
    let oauth_token =
        std::env::var("YOUTUBE_OAUTH_TOKEN").context("YOUTUBE_OAUTH_TOKEN not set")?;

    //let playlist_id = std::env::var("PLAYLIST_ID").context("PLAYLIST_ID not set")?;

    print!("YouTube playlist: ");
    let playlist_id: String = {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Couldn't get input");
        let parsed = Url::parse(&buf).context("Invalid URL")?;
        parsed
            .query_pairs()
            .find(|(key, _)| key == "list")
            .map(|(_, value)| value.into_owned())
            .context("No playlist ID found in URL")?
    };

    print!("How would you like to sort these videos? ");
    let order: SortOrder = {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Couldn't get input");
        SortOrder::iter()
            .map(|x| (x.to_string().to_lowercase(), x))
            .collect::<HashMap<_, _>>()[&buf.to_lowercase()]
    };

    let client = Client::new();

    println!("Fetching playlist items for {playlist_id}...",);
    let mut items = fetch_all_items(&client, &playlist_id, &api_key, &oauth_token).await?;
    println!("  {} items fetched.", items.len());

    let durations = fetch_durations(&client, &items, &api_key, &oauth_token).await?;
    println!("  Durations fetched.");

    println!("Sorting ({:?})…", order);
    sort_items(&mut items, order, &durations);

    draw_chart(&durations.into_values().collect());

    println!("Pushing new order…");
    push_new_order(&client, &items, &playlist_id, &api_key, &oauth_token).await?;

    Ok(())
}

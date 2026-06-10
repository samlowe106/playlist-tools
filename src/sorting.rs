use std::collections::HashMap;

use crate::models::PlaylistItem;

use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Display, clap::ValueEnum, Default)]
pub enum SortOrder {
    /// Title of the video
    Title,
    /// Date the video was published
    Published,
    /// Duration of the YouTube video
    #[default]
    Duration,
    /// Name of the YouTube channel that uploaded it
    UploaderName,
    // Reverse the current playlist order
    //Reversed,
}

pub fn sort_items(
    items: &mut [PlaylistItem],
    order: SortOrder,
    durations: &HashMap<String, u64>,
    asending: bool,
) {
    /* Sort */
    match order {
        SortOrder::Title => {
            items.sort_by_cached_key(|i| {
                i.snippet.title.to_lowercase()
                //.cmp(&b.snippet.title.to_lowercase())
            });
        }
        SortOrder::Published => {
            // ISO 8601 strings sort lexicographically
            //  no parsing needed
            items.sort_by(|a, b| {
                a.content_details
                    .video_published_at
                    .as_deref()
                    .unwrap_or("")
                    .cmp(
                        b.content_details
                            .video_published_at
                            .as_deref()
                            .unwrap_or(""),
                    )
            });
        }
        SortOrder::Duration => {
            items.sort_by_cached_key(|i| {
                durations
                    .get(&i.content_details.video_id)
                    .copied()
                    .unwrap_or(0)
            });
        }
        SortOrder::UploaderName => {
            items.sort_by(|a, b| {
                a.snippet
                    .video_owner_channel_title
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b.snippet.video_owner_channel_title.as_deref().unwrap_or(""))
            });
        }
    };

    if asending {
        items.reverse();
    };
}

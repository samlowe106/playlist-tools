use std::collections::HashMap;

use crate::models::PlaylistItem;

use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, EnumIter, Display)]
pub enum SortOrder {
    /// Title of the video
    Title,
    /// Date the video was published
    Published,
    /// Duration of the YouTube video
    Duration,
    /// Name of the YouTube channel that uploaded it
    UploaderName,
    /// Reverse the current playlist order
    Reversed,
}

/* stackoverflow
impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}
*/

pub fn sort_items(
    items: &mut Vec<PlaylistItem>,
    order: SortOrder,
    durations: &HashMap<String, u64>,
) {
    /* Sort */
    match order {
        SortOrder::Title => {
            items.sort_by(|a, b| {
                a.snippet
                    .title
                    .to_lowercase()
                    .cmp(&b.snippet.title.to_lowercase())
            });
        }
        SortOrder::Published => {
            // ISO 8601 strings sort lexicographically — no parsing needed
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
            items.sort_by_key(|i| {
                durations
                    .get(&i.content_details.video_id)
                    .copied()
                    .unwrap_or(0)
            });
        }
        SortOrder::UploaderName => {
            items.sort_by(|a, b| todo!());
        }
        SortOrder::Reversed => {
            items.reverse();
        }
    }
}

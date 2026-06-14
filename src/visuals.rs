use plotters::{prelude::*, style::full_palette::ORANGE};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum GraphOption {
    Durations,
    Uploaders,
}

pub struct ChartConfig<'a> {
    pub title: &'a str,
    pub x_desc: &'a str,
    pub y_desc: &'a str,
    pub output_path: &'a str,
}

fn draw_histogram(
    config: ChartConfig,
    labels: Vec<String>,
    counts: Vec<u32>,
) -> anyhow::Result<()> {
    let max_count = counts.iter().copied().max().unwrap_or(1) + 1;
    let num_bins = counts.len() as u32;

    let root = SVGBackend::new(config.output_path, (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(config.title, ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(40)
        .build_cartesian_2d(0u32..num_bins, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_desc(config.x_desc)
        .y_desc(config.y_desc)
        .x_label_formatter(&|x| labels.get(*x as usize).cloned().unwrap_or_default())
        .x_labels(10)
        .draw()?;

    chart
        .draw_series(counts.iter().enumerate().map(|(i, &count)| {
            Rectangle::new([(i as u32, 0), (i as u32 + 1, count)], ORANGE.filled())
        }))?
        .label(config.x_desc)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], ORANGE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!("Chart saved to {}", config.output_path);
    Ok(())
}

pub fn draw_durations_chart(title: &str, durations: &[u64]) -> anyhow::Result<()> {
    let bin_size_mins = 5u64;
    let durations_mins: Vec<u64> = durations.iter().map(|&d| d / 60).collect();
    let max_mins = durations_mins.iter().copied().max().unwrap_or_default();
    let num_bins = (max_mins / bin_size_mins + 1) as usize;

    let mut counts = vec![0u32; num_bins];
    for &d in &durations_mins {
        counts[(d / bin_size_mins) as usize] += 1;
    }

    let labels = (0..num_bins)
        .map(|i| format!("{}", i as u64 * bin_size_mins))
        .collect();

    draw_histogram(
        ChartConfig {
            title,
            x_desc: "Duration (minutes)",
            y_desc: "Videos",
            output_path: "durations.svg",
        },
        labels,
        counts,
    )
}

pub fn draw_uploaders_chart(title: &str, uploaders: &[String]) -> anyhow::Result<()> {
    // Count occurrences of each uploader
    let mut counts_map: HashMap<&str, u32> = HashMap::new();
    for u in uploaders {
        *counts_map.entry(u.as_str()).or_insert(0) += 1;
    }

    // Sort by count descending so the most prolific uploaders are first
    let mut pairs: Vec<(&str, u32)> = counts_map.into_iter().collect();
    pairs.sort_by_key(|b| std::cmp::Reverse(b.1));

    let labels = pairs.iter().map(|(name, _)| name.to_string()).collect();
    let counts = pairs.iter().map(|(_, count)| *count).collect();

    draw_histogram(
        ChartConfig {
            title,
            x_desc: "Uploader",
            y_desc: "Videos",
            output_path: "uploaders.svg",
        },
        labels,
        counts,
    )
}

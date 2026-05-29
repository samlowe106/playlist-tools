use std::vec::Vec;

//use crate::models::{PlaylistItem, PlaylistItemListResponse, UpdateBody, UpdateSnippet};
use plotters::{prelude::*, style::full_palette::ORANGE};

pub fn draw_chart(durations: &Vec<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new("durations.svg", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_secs = durations.iter().copied().max().unwrap_or_default(); // durations.iter().map(|d| d.as_secs()).max().unwrap_or(0)
    let max_count = 20u32; // TODO: calculate from bin counts

    let mut chart = ChartBuilder::on(&root)
        .caption("Video durations", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u64..max_secs, 0u32..max_count)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(ORANGE.filled())
                .margin(1)
                .data(durations.iter().map(|&x| (x, 1))),
        )?
        .label("y = x^2")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}

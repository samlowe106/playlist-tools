use plotters::{prelude::*, style::full_palette::ORANGE};

pub fn draw_chart(durations: &[u64]) -> anyhow::Result<()> {
    let root = SVGBackend::new("durations.svg", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    // Convert to minutes and use 5-minute bins
    let durations_mins: Vec<u64> = durations.iter().map(|&d| d / 60).collect();
    let max_mins = durations_mins.iter().copied().max().unwrap_or_default() + 1;

    let bin_size = 5u64; // 5 minute bins
    let num_bins = (max_mins / bin_size + 1) as usize;
    let mut counts = vec![0u32; num_bins];
    for &d in &durations_mins {
        counts[(d / bin_size) as usize] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1) + 1;

    let mut chart = ChartBuilder::on(&root)
        .caption("Video durations", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u64..max_mins, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_desc("Duration (minutes)")
        .y_desc("Videos")
        .draw()?;

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(ORANGE.filled())
                .margin(1)
                .data(durations_mins.iter().map(|&x| (x, 1u32))),
        )?
        .label("Duration")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], ORANGE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!("Chart saved to durations.svg");
    Ok(())
}

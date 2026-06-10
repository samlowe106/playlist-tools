use plotters::{prelude::*, style::full_palette::ORANGE};

pub fn draw_chart(durations: &[u64]) -> anyhow::Result<()> {
    let root = SVGBackend::new("durations.svg", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_secs = durations.iter().copied().max().unwrap_or_default();

    // Calculate the actual max bin count so no bars get clipped
    let bin_size = 60u64; // 1 minute bins
    let mut counts = vec![0u32; (max_secs / bin_size + 1) as usize];
    for &d in durations {
        counts[(d / bin_size) as usize] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1) + 1; // +1 for breathing room

    let mut chart = ChartBuilder::on(&root)
        .caption("Video durations", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u64..max_secs, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_desc("Duration (seconds)")
        .y_desc("Videos")
        .draw()?;

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(ORANGE.filled())
                .margin(1)
                .data(durations.iter().map(|&x| (x, 1u32))),
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

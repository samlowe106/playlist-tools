use plotters::{prelude::*, style::full_palette::ORANGE};

pub fn draw_durations_chart(title: &str, durations: &[u64]) -> anyhow::Result<()> {
    let root = SVGBackend::new("durations.svg", (800, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let bin_size_mins = 5u64;
    let durations_mins: Vec<u64> = durations.iter().map(|&d| d / 60).collect();
    let max_mins = durations_mins.iter().copied().max().unwrap_or_default();
    let num_bins = (max_mins / bin_size_mins + 1) as usize;

    // Pre-bucket the data
    let mut counts = vec![0u32; num_bins];
    for &d in &durations_mins {
        counts[(d / bin_size_mins) as usize] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1) + 1;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0u32..num_bins as u32, 0u32..max_count)?;

    chart
        .configure_mesh()
        .x_desc("Duration (minutes)")
        .y_desc("Videos")
        .x_label_formatter(&|x| format!("{}", x * bin_size_mins as u32))
        .draw()?;

    // Draw bars directly from counts using bin index as x
    chart
        .draw_series(counts.iter().enumerate().map(|(i, &count)| {
            let x0 = i as u32;
            let x1 = x0 + 1;
            Rectangle::new([(x0, 0), (x1, count)], ORANGE.filled())
        }))?
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

use std::fs::File;
use std::io::Write;
use plotters::prelude::*;
use anyhow::Result;

#[derive(Debug)]
pub struct DataPoint {
    pub velocity_fraction: f64,
    pub gamma: f64,
    pub proper_time: f64,
    pub dilated_time: f64,
    pub proper_length: f64,
    pub contracted_length: f64,
}

pub fn export_csv(log: &[DataPoint]) -> std::io::Result<()> {
    let mut file = File::create("metrics.csv")?;
    writeln!(
        file,
        "velocity_fraction,gamma,proper_time,dilated_time,proper_length,contracted_length"
    )?;
    for dp in log {
        writeln!(
            file,
            "{:.3},{:.6},{:.3},{:.3},{:.3},{:.3}",
            dp.velocity_fraction,
            dp.gamma,
            dp.proper_time,
            dp.dilated_time,
            dp.proper_length,
            dp.contracted_length,
        )?;
    }
    Ok(())
}

pub fn plot_results(log: &[DataPoint]) -> Result<()> {
    let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Relativity Effects", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..1f64, 0f64..(log.iter().map(|d| d.dilated_time).fold(0./0., f64::max)))?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            log.iter().map(|d| (d.velocity_fraction, d.dilated_time)),
            &BLUE,
        ))?
        .label("Time Dilation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

    chart
        .draw_series(LineSeries::new(
            log.iter().map(|d| (d.velocity_fraction, d.contracted_length)),
            &RED,
        ))?
        .label("Length Contraction")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

    chart
        .draw_series(LineSeries::new(
            log.iter().map(|d| (d.velocity_fraction, d.gamma)),
            &GREEN,
        ))?
        .label("Lorentz Factor γ")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &GREEN));

    chart.configure_series_labels().border_style(&BLACK).draw()?;

    Ok(())
}


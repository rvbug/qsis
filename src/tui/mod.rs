use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io;

use plotters::prelude::*;

use crate::relativity::special::{lorentz_factor, C, length_contraction };

#[derive(Debug)]
struct DataPoint {
    velocity_fraction: f64,
    gamma: f64,
    proper_time: f64,
    dilated_time: f64,
    proper_length: f64,
    contracted_length: f64,
}


pub fn start() -> anyhow::Result<()> {
    // Setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State
    let mut velocity_fraction: f64 = 0.0;
    let proper_time: f64 = 10.0;      // years
    let proper_length: f64 = 100.0;   // meters  ✅ ADD THIS
    let mut log: Vec<DataPoint> = Vec::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let v = velocity_fraction * C;
            let gamma = lorentz_factor(v);
            let dilated_time = proper_time * gamma;
            let contracted_length = length_contraction(proper_length, v);  // ✅ fixed arg order

            let velocity_text = format!("Velocity: {:.2}c", velocity_fraction);
            let gamma_text = format!("Lorentz factor (γ): {:.4}", gamma);
            let time_text = format!(
                "Proper time: {:.1} years | Dilated time: {:.2} years",
                proper_time, dilated_time
            );
            let length_text = format!(
                "Proper length: {:.1} m | Contracted length: {:.2} m",
                proper_length, contracted_length
            );

            let blocks = vec![
                Paragraph::new(velocity_text).block(Block::default().borders(Borders::ALL)),
                Paragraph::new(gamma_text).block(Block::default().borders(Borders::ALL)),
                Paragraph::new(time_text).block(Block::default().borders(Borders::ALL)),
                Paragraph::new(length_text).block(Block::default().borders(Borders::ALL)),
            ];

            for (i, b) in blocks.into_iter().enumerate() {
                f.render_widget(b, chunks[i]);
            }
        })?;

        // Input handling
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right => {
                        if velocity_fraction < 0.99 {
                            velocity_fraction += 0.01;
                        }
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));  // ✅ fixed
                    }
                    KeyCode::Left => {
                        if velocity_fraction > 0.0 {
                            velocity_fraction -= 0.01;
                        }
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));  // ✅ fixed
                    }
                    KeyCode::Char('q') => {
                        crossterm::terminal::disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        export_csv(&log)?;
                        plot_results(&log)?;

                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}



fn snapshot(velocity_fraction: f64, proper_time: f64, proper_length: f64) -> DataPoint {
    let v = velocity_fraction * C;
    let gamma = lorentz_factor(v);
    let dilated_time = proper_time * gamma;
    let contracted_length = length_contraction(proper_length, v);

    DataPoint {
        velocity_fraction,
        gamma,
        proper_time,
        dilated_time,
        proper_length,
        contracted_length,
    }
}

fn export_csv(log: &Vec<DataPoint>) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create("realtime.csv")?;
    writeln!(file, "velocity_fraction,gamma,proper_time,dilated_time")?;

    for entry in log {
        writeln!(
            file,
            "{:.2},{:.6},{:.1},{:.6}",
            entry.velocity_fraction, entry.gamma, entry.proper_time, entry.dilated_time
        )?;
    }
    println!("✅ Data exported to realtime.csv");
    Ok(())
}

fn plot_results(log: &Vec<DataPoint>) -> anyhow::Result<()> {
    let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let y_max = log.iter().map(|d| d.dilated_time).fold(0.0, f64::max).ceil();

    let mut chart = ChartBuilder::on(&root)
        .caption("Time Dilation vs Velocity", ("sans-serif", 24))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..1f64, 0f64..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Velocity (fraction of c)")
        .y_desc("Dilated Time (years)")
        .draw()?;

    let series: Vec<(f64, f64)> = log
        .iter()
        .map(|d| (d.velocity_fraction, d.dilated_time))
        .collect();

    chart.draw_series(LineSeries::new(series, &RED))?;
    println!("✅ Plot saved to plot.png");
    Ok(())
}


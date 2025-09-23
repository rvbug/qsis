use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
    Terminal,
};

use crate::relativity::special::{lorentz_factor, length_contraction, C};
use crate::metrics::{DataPoint, export_csv, plot_results};

/// Chart display modes
enum ChartMode {
    All,
    TimeDilation,
    LengthContraction,
    LorentzFactor,
}

pub fn start() -> anyhow::Result<()> {
    // Setup terminal
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Simulation state
    let mut velocity_fraction: f64 = 0.0;
    let proper_time: f64 = 10.0;      // years
    let proper_length: f64 = 100.0;   // meters
    let mut log: Vec<DataPoint> = Vec::new();
    let mut chart_mode = ChartMode::All;

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(2),
                        Constraint::Length(2),
                        Constraint::Min(10),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Current values
            let v = velocity_fraction * C;
            let gamma = lorentz_factor(v);
            let dilated_time = proper_time * gamma;
            let contracted_length = length_contraction(proper_length, v);

            // Top stats display
            let stats = vec![
                Paragraph::new(format!("Velocity: {:.2}c", velocity_fraction))
                    .block(Block::default().borders(Borders::ALL)),
                Paragraph::new(format!("Lorentz factor (γ): {:.4}", gamma))
                    .block(Block::default().borders(Borders::ALL)),
                Paragraph::new(format!(
                    "Time: proper = {:.1} y | dilated = {:.2} y",
                    proper_time, dilated_time
                ))
                .block(Block::default().borders(Borders::ALL)),
            ];

            for (i, stat) in stats.into_iter().enumerate() {
                f.render_widget(stat, chunks[i]);
            }


            // Precompute all datasets safely
            let gamma_data: Vec<(f64, f64)> = log.iter()
                .map(|d| (d.velocity_fraction, d.gamma))
                .collect();
            let time_data: Vec<(f64, f64)> = log.iter()
                .map(|d| (d.velocity_fraction, d.dilated_time))
                .collect();
            let length_data: Vec<(f64, f64)> = log.iter()
                .map(|d| (d.velocity_fraction, d.contracted_length))
                .collect();

            let datasets = match chart_mode {
                ChartMode::All => vec![
                    Dataset::default()
                        .name("γ (Lorentz)")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&gamma_data),
                    Dataset::default()
                        .name("Time Dilation")
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&time_data),
                    Dataset::default()
                        .name("Length Contraction")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Magenta))
                        .data(&length_data),
                ],
                ChartMode::TimeDilation => vec![Dataset::default()
                    .name("Time Dilation")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&time_data)],
                ChartMode::LengthContraction => vec![Dataset::default()
                    .name("Length Contraction")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Magenta))
                    .data(&length_data)],
                ChartMode::LorentzFactor => vec![Dataset::default()
                    .name("γ (Lorentz)")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&gamma_data)],
            };



            // Chart widget
            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title("Relativity Visualisation (a=all, t=time, l=length, g=γ, q=quit)")
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Velocity (c)")
                        .bounds([0.0, 1.0]),
                )
                .y_axis(
                    Axis::default()
                        .title("Values")
                        .bounds([0.0, proper_time * 10.0]),
                );

            f.render_widget(chart, chunks[3]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Right => {
                        if velocity_fraction < 0.99 {
                            velocity_fraction += 0.01;
                        }
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));
                    }
                    KeyCode::Left => {
                        if velocity_fraction > 0.0 {
                            velocity_fraction -= 0.01;
                        }
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));
                    }
                    KeyCode::Char('a') => chart_mode = ChartMode::All,
                    KeyCode::Char('t') => chart_mode = ChartMode::TimeDilation,
                    KeyCode::Char('l') => chart_mode = ChartMode::LengthContraction,
                    KeyCode::Char('g') => chart_mode = ChartMode::LorentzFactor,
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


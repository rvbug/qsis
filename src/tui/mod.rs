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
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, Paragraph},
    Terminal,
};

use crate::relativity::special::{lorentz_factor, length_contraction, C};
use crate::metrics::{DataPoint, export_csv, plot_results};

/// Which dataset(s) to show in the chart
enum ChartMode {
    All,
    TimeDilation,
    LengthContraction,
    LorentzFactor,
}

pub fn start() -> anyhow::Result<()> {
    // Terminal setup: alternate screen + raw mode
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Simulation state
    let mut velocity_fraction: f64 = 0.0;
    let proper_time: f64 = 10.0; // years
    let proper_length: f64 = 100.0; // meters
    let mut log: Vec<DataPoint> = Vec::new();
    let mut chart_mode = ChartMode::All;

    // push initial sample so chart isn't empty
    log.push(snapshot(velocity_fraction, proper_time, proper_length));

    // Main loop
    loop {
        terminal.draw(|f| {
            // area / layout
            // note: older ratatui used `.size()`; if you have deprecation warnings you can use `.area()` depending on your ratatui version
            let area = f.size();
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
                .split(area);

            // current values
            let v = velocity_fraction * C;
            let gamma = lorentz_factor(v);
            let dilated_time = proper_time * gamma;
            let contracted_length = length_contraction(proper_length, v);

            // top / small stats
            let stat_left = Paragraph::new(format!("Velocity: {:.2} c", velocity_fraction))
                .block(Block::default().borders(Borders::ALL));
            let stat_mid = Paragraph::new(format!("γ (Lorentz): {:.4}", gamma))
                .block(Block::default().borders(Borders::ALL));
            let stat_right = Paragraph::new(format!(
                "Time: proper = {:.1} y | dilated = {:.2} y",
                proper_time, dilated_time
            ))
            .block(Block::default().borders(Borders::ALL));

            f.render_widget(stat_left, chunks[0]);
            f.render_widget(stat_mid, chunks[1]);
            f.render_widget(stat_right, chunks[2]);

            // --- build owned data arrays so references live long enough ---
            let gamma_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.gamma)).collect();
            let time_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.dilated_time)).collect();
            let length_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.contracted_length)).collect();

            // choose datasets based on mode
            let datasets = match chart_mode {
                ChartMode::All => vec![
                    Dataset::default()
                        .name("γ (Lorentz)")
                        .marker(symbols::Marker::Braille)
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
                ChartMode::TimeDilation => vec![
                    Dataset::default()
                        .name("Time Dilation")
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Cyan))
                        .data(&time_data),
                ],
                ChartMode::LengthContraction => vec![
                    Dataset::default()
                        .name("Length Contraction")
                        .marker(symbols::Marker::Dot)
                        .style(Style::default().fg(Color::Magenta))
                        .data(&length_data),
                ],
                ChartMode::LorentzFactor => vec![
                    Dataset::default()
                        .name("γ (Lorentz)")
                        .marker(symbols::Marker::Braille)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&gamma_data),
                ],
            };

            // dynamic y-axis max (safe fallback to 1.0)
            let y_max = {
                let max_val = match chart_mode {
                    ChartMode::All => {
                        gamma_data.iter().map(|(_, y)| *y)
                            .chain(time_data.iter().map(|(_, y)| *y))
                            .chain(length_data.iter().map(|(_, y)| *y))
                            .fold(1.0_f64, f64::max)
                    }
                    ChartMode::TimeDilation => time_data.iter().map(|(_, y)| *y).fold(1.0_f64, f64::max),
                    ChartMode::LengthContraction => length_data.iter().map(|(_, y)| *y).fold(1.0_f64, f64::max),
                    ChartMode::LorentzFactor => gamma_data.iter().map(|(_, y)| *y).fold(1.0_f64, f64::max),
                };
                // if max_val is <= 0, fallback to 1.0
                if max_val <= 0.0 { 1.0 } else { max_val }
            };

            // chart widget with legend hint in title (instructions)
            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title("Relativity Visualization — (a:all, t:time, l:length, g:γ, ←/→:change v, q:quit)")
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Velocity (fraction of c)")
                        .bounds([0.0, 1.0]),
                )
                .y_axis(
                    Axis::default()
                        .title("Value")
                        .bounds([0.0, y_max * 1.1]), // 10% padding
                );

            f.render_widget(chart, chunks[3]);
        })?;

        // input / events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => {
                        velocity_fraction = (velocity_fraction - 0.01).max(0.0);
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));
                    }
                    KeyCode::Right => {
                        velocity_fraction = (velocity_fraction + 0.01).min(0.99);
                        log.push(snapshot(velocity_fraction, proper_time, proper_length));
                    }
                    KeyCode::Char('a') => chart_mode = ChartMode::All,
                    KeyCode::Char('t') => chart_mode = ChartMode::TimeDilation,
                    KeyCode::Char('l') => chart_mode = ChartMode::LengthContraction,
                    KeyCode::Char('g') => chart_mode = ChartMode::LorentzFactor,
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // clean up terminal first
                        crossterm::terminal::disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        // export and plot using metrics helpers
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

/// create DataPoint from state
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


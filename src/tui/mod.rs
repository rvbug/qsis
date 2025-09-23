use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
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

enum ChartMode {
    All,
    TimeDilation,
    LengthContraction,
    LorentzFactor,
}

enum RunMode {
    Auto,
    Manual(Vec<f64>),
    Interactive,
}

pub fn start() -> anyhow::Result<()> {
    // Ask mode
    println!("Select mode:");
    println!("1. Auto-simulation (sweep 0 → 0.99c)");
    println!("2. Manual input (e.g., 0.3,0.5,0.9)");
    println!("3. Interactive (arrow keys)");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let mode = match input.trim() {
        "1" => RunMode::Auto,
        "2" => {
            println!("Enter comma-separated velocities (fractions of c, e.g. 0.3,0.6,0.9):");
            let mut vals = String::new();
            io::stdin().read_line(&mut vals)?;
            let parsed: Vec<f64> = vals
                .trim()
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();
            RunMode::Manual(parsed)
        }
        _ => RunMode::Interactive,
    };

    // Terminal setup
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut velocity_fraction: f64 = 0.0;
    let proper_time: f64 = 10.0;
    let proper_length: f64 = 100.0;
    let mut log: Vec<DataPoint> = Vec::new();
    let mut chart_mode = ChartMode::All;

    // Pre-fill log for Auto/Manual
    match &mode {
        RunMode::Auto => {
            for v in (0..100).map(|i| i as f64 / 100.0) {
                log.push(snapshot(v, proper_time, proper_length));
            }
        }
        RunMode::Manual(vals) => {
            for &v in vals {
                if v >= 0.0 && v < 1.0 {
                    log.push(snapshot(v, proper_time, proper_length));
                }
            }
        }
        RunMode::Interactive => {
            log.push(snapshot(velocity_fraction, proper_time, proper_length));
        }
    }

    loop {
        terminal.draw(|f| {
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

            let v = velocity_fraction * C;
            let gamma = lorentz_factor(v);
            let dilated_time = proper_time * gamma;
            let contracted_length = length_contraction(proper_length, v);

            let stat1 = Paragraph::new(format!("Velocity: {:.2} c", velocity_fraction))
                .block(Block::default().borders(Borders::ALL));
            let stat2 = Paragraph::new(format!("γ (Lorentz): {:.4}", gamma))
                .block(Block::default().borders(Borders::ALL));
            let stat3 = Paragraph::new(format!(
                "Time: proper = {:.1} y | dilated = {:.2} y | Lc = {:.1} m",
                proper_time, dilated_time, contracted_length
            ))
            .block(Block::default().borders(Borders::ALL));

            f.render_widget(stat1, chunks[0]);
            f.render_widget(stat2, chunks[1]);
            f.render_widget(stat3, chunks[2]);

            // Owned data for safe lifetimes
            let gamma_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.gamma)).collect();
            let time_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.dilated_time)).collect();
            let length_data: Vec<(f64, f64)> = log.iter().map(|d| (d.velocity_fraction, d.contracted_length)).collect();

            let datasets = match chart_mode {
                ChartMode::All => vec![
                    Dataset::default().name("γ").style(Style::default().fg(Color::Yellow)).data(&gamma_data),
                    Dataset::default().name("Time Dilation").style(Style::default().fg(Color::Cyan)).data(&time_data),
                    Dataset::default().name("Length Contraction").style(Style::default().fg(Color::Magenta)).data(&length_data),
                ],
                ChartMode::TimeDilation => vec![
                    Dataset::default().name("Time Dilation").style(Style::default().fg(Color::Cyan)).data(&time_data),
                ],
                ChartMode::LengthContraction => vec![
                    Dataset::default().name("Length Contraction").style(Style::default().fg(Color::Magenta)).data(&length_data),
                ],
                ChartMode::LorentzFactor => vec![
                    Dataset::default().name("γ").style(Style::default().fg(Color::Yellow)).data(&gamma_data),
                ],
            };

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title("Relativity Visualization (a=all, t=time, l=length, g=γ, q=quit)")
                        .borders(Borders::ALL),
                )
                .x_axis(Axis::default().title("Velocity (c)").bounds([0.0, 1.0]))
                .y_axis(Axis::default().title("Value").bounds([0.0, 500.0]));

            f.render_widget(chart, chunks[3]);
        })?;

        // Auto / Manual do not require key events
        if let RunMode::Interactive = mode {
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
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        } else {
            // In auto/manual modes, just wait for q
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Export CSV + plot
    export_csv(&log)?;
    plot_results(&log)?;

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


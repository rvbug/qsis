use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

const C: f64 = 299_792_458.0; // speed of light in m/s

fn lorentz_factor(v: f64) -> f64 {
    1.0 / (1.0 - (v * v) / (C * C)).sqrt()
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    // Simulation state
    let mut velocity_fraction: f64 = 0.0; // in units of c
    let proper_time: f64 = 10.0; // years

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [Constraint::Length(3), Constraint::Length(3), Constraint::Length(3)]
                        .as_ref(),
                )
                .split(size);

            let v = velocity_fraction * C;
            let gamma = lorentz_factor(v);
            let dilated_time = proper_time * gamma;

            let velocity_text =
                format!("Velocity: {:.2}c", velocity_fraction);
            let gamma_text = format!("Lorentz factor (γ): {:.4}", gamma);
            let time_text = format!(
                "Proper time: {:.1} years | Dilated time: {:.2} years",
                proper_time, dilated_time
            );

            let blocks = vec![
                Paragraph::new(velocity_text).block(Block::default().borders(Borders::ALL)),
                Paragraph::new(gamma_text).block(Block::default().borders(Borders::ALL)),
                Paragraph::new(time_text).block(Block::default().borders(Borders::ALL)),
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
                    }
                    KeyCode::Left => {
                        if velocity_fraction > 0.0 {
                            velocity_fraction -= 0.01;
                        }
                    }
                    KeyCode::Char('q') => {
                        crossterm::terminal::disable_raw_mode()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}


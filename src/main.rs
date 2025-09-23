mod relativity;
mod tui;
mod metrics;

use clap::{Parser, Subcommand};
use relativity::special::lorentz_factor;
use anyhow::Result;

/// QSIS - Quantum Spacetime Intelligence System
#[derive(Parser)]
#[command(name = "qsis", about = "Rust framework for computational time travel")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive TUI simulation
    Tui,
    /// Generate metrics and export to CSV
    Metrics,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Tui) => tui::start()?,
        Some(Commands::Metrics) => run_metrics()?,
        None => tui::start()?, // default
    }

    Ok(())
}

fn run_metrics() -> anyhow::Result<()> {
    // use std::fs::File;
    // use std::io::Write;
    // let mut file = File::create("metrics.csv")?;
    // writeln!(file, "velocity_fraction,gamma,proper_time,dilated_time, proper_length, contracted_length")?;

    let proper_time = 10.0; // years
    let proper_length = 100.0; // meters
    
    for i in 0..100 {
        let v_frac = i as f64 / 100.0;
        let v = v_frac * relativity::special::C;
        let gamma = lorentz_factor(v);
        // let dilated_time = proper_time * gamma;
        // let contracted_length = relativity::special::length_contraction(proper_length, v);
    //     writeln!(
    //     file,
    //     "{:.2},{:.6},{:.1},{:.6},{:.1},{:.6}",
    //     v_frac, gamma, proper_time, dilated_time, proper_length, contracted_length
    // )?;

    }

    println!("✅ Metrics written to metrics.csv");
    Ok(())
}


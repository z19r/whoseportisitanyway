use anyhow::Result;
use clap::{Parser, Subcommand};

use whoseportisitanyway::cli;
use whoseportisitanyway::config::Config;
use whoseportisitanyway::tui;

#[derive(Parser)]
#[command(
    name = "whoseportisitanyway",
    about = "Which ports are in use, who owns them, and is it your dev server or something blocking it?"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print a one-shot port table
    Snapshot {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Explain what's using a specific port
    Why {
        /// Port number to investigate
        port: u16,
    },
    /// List all ports (JSON by default)
    List {
        /// Output as tab-separated plain text
        #[arg(long)]
        plain: bool,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Config::load()?;

    match args.command {
        None => tui::run(&config),
        Some(Command::Snapshot { json }) => cli::snapshot::run(&config, json),
        Some(Command::Why { port }) => cli::why::run(&config, port),
        Some(Command::List { plain }) => cli::list::run(&config, plain),
    }
}

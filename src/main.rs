use under::main_handler;

use ansi_term::Color::*;
use clap::{builder::styling::*, Parser, Subcommand};
use std::path::PathBuf;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Yellow.on_default())
}

#[derive(Parser)]
#[command(
    version,
    about = format!("{}",
        Purple.bold().paint("Underivability Explorations"),
    ),
    long_about = None,
    styles = styles(),
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run Honeybee interactively in the CLI
    Interact {
        /// The AND-OR graph to use (.json)
        #[arg(short, long, value_name = "FILE")]
        graph: PathBuf,
    },
}

impl Command {
    pub fn handle(self) -> Result<(), String> {
        match self {
            Self::Interact { graph } => main_handler::interact(&graph),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result = cli.command.handle();

    match result {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}

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
    /// Run interactively in the CLI
    Interact {
        /// The AND-OR graph to use (.json)
        #[arg(short, long, value_name = "FILE")]
        graph: PathBuf,
    },

    /// Run a benchmark suite
    Benchmark {
        /// The path to the benchmark suite directory
        #[arg(short, long, value_name = "DIRECTORY")]
        path: PathBuf,
    },

    /// Convert various representations to the AND/OR JSON Graph Format
    Convert {
        /// The file to convert
        #[arg(short, long, value_name = "FILE")]
        path: PathBuf,

        /// The format to convert from
        #[arg(short, long, value_name = "FORMAT")]
        format: main_handler::ConversionInputFormat,
    },
}

impl Command {
    pub fn handle(self) -> Result<(), String> {
        match &self {
            Self::Interact { graph } => main_handler::interact(graph),
            Self::Benchmark { path } => main_handler::benchmark(path),
            Self::Convert { path, format } => {
                main_handler::convert(path, format)
            }
        }
    }
}

fn main() {
    env_logger::init();

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

use under::main_handler;
use under::menu;

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

// fn providers_parser(s: &str) -> Result<Vec<menu::Provider>, String> {
//     let mut ret = vec![];
//     for p in s.split(",") {
//         ret.push(p.try_into()?)
//     }
//     Ok(ret)
// }

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
        #[arg(value_name = "FILE")]
        graph: PathBuf,

        /// Comma-separated list of providers to (options: CA, CR, ASC, R)
        #[arg(
            short,
            long,
            value_name = "PROVIDERS",
            num_args = 1..,
            value_delimiter = ',',
            default_value = "CA,CR")
        ]
        providers: Vec<menu::Provider>,
    },

    /// Run a benchmark suite
    Benchmark {
        /// The path to the benchmark suite directory
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,
    },

    /// Generate solutions for a benchmark suite
    GenerateSolutions {
        /// The path to the benchmark suite directory
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,
    },

    /// Convert various representations to the AND/OR JSON Graph Format
    Convert {
        /// The file to convert
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// The format to convert from (options: EGraphSerialize, AOJsonGraph, Argus)
        #[arg(short, long, value_name = "FORMAT")]
        format: main_handler::ConversionInputFormat,
    },

    /// Render an AND/OR graph in the AND/OR JSON Graph Format (stored in out/RENDERED.dot and out/RENDERED.pdf)
    Render {
        /// The path to the graph to render
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
}

impl Command {
    pub fn handle(self) -> Result<(), String> {
        match &self {
            Self::Interact { graph, providers } => {
                main_handler::interact(graph, providers)
            }
            Self::Benchmark { path } => main_handler::benchmark(path),
            Self::GenerateSolutions { path } => {
                main_handler::generate_solutions(path)
            }
            Self::Convert { path, format } => {
                main_handler::convert(path, format)
            }
            Self::Render { path } => main_handler::render(path),
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

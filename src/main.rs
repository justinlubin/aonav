use aonav::main_handler;
use aonav::menu;

use ansi_term::Color::*;
use clap::{builder::styling::*, Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;

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
        #[arg(value_name = "FILE")]
        graph: PathBuf,

        /// Comma-separated list of providers
        #[arg(
            short,
            long,
            value_name = "PROVIDERS",
            value_delimiter = ',',
            default_value = "AlphabeticalRelevant"
        )]
        providers: Vec<menu::Provider>,

        /// Reduce the input before interacting (perform unit propagation)
        #[arg(long, action)]
        reduce: bool,

        /// Use incrementality (if possible)
        #[arg(long, action)]
        incremental: bool,

        /// Additionally render a PDF in out/INTERACTIVE.pdf
        #[arg(long, action)]
        pdf: bool,
    },

    /// Run a benchmark suite
    Benchmark {
        /// The path to the benchmark suite directory
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,

        /// Comma-separated list of providers to benchmark
        #[arg(
            short,
            long,
            value_name = "PROVIDERS",
            value_delimiter = ',',
            default_value = "Remaining"
        )]
        providers: Vec<menu::Provider>,

        /// The path to the benchmark suite directory
        #[arg(short, long, default_value_t = 5)]
        replicates: usize,

        /// Run the benchmark entries in parallel
        #[arg(long, action)]
        parallel: bool,

        /// Use the minimal solutions
        #[arg(long, action)]
        minimal: bool,

        /// Use incrementality (if possible)
        #[arg(long, action)]
        incremental: bool,

        /// Timeout (in seconds, omit for none)
        #[arg(long)]
        timeout: Option<u64>,

        /// Use validity stopping (instead of sufficiency)
        #[arg(long, action)]
        stop_on_valid: bool,

        /// Count decisions as cardinality of options presented (unordered output)
        #[arg(long, action)]
        count_unordered: bool,
    },

    /// Emit graph statistics for a benchmark suite
    BenchmarkStats {
        /// The path to the benchmark suite directory
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,
    },

    /// Generate solutions for a benchmark suite
    GenerateSolutions {
        /// The number of solutions to generate per graph
        #[arg(short, long, value_name = "DIRECTORY", default_value_t = 10)]
        count: usize,

        /// The path to the benchmark suite directory
        #[arg(value_name = "DIRECTORY")]
        path: PathBuf,

        /// Run in parallel
        #[arg(long, action)]
        parallel: bool,
    },

    /// Convert various representations to the AND/OR JSON Graph Format
    Convert {
        /// The file to convert
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// The format to convert from
        #[arg(short, long, value_name = "FORMAT")]
        format: main_handler::ConversionInputFormat,

        /// Randomize the IDs of the output
        #[arg(long, action)]
        randomize: bool,

        /// Reduce the output (perform unit propagation)
        #[arg(long, action)]
        reduce: bool,
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
            Self::Interact {
                graph,
                providers,
                reduce,
                incremental,
                pdf,
            } => main_handler::interact(
                graph,
                providers,
                *reduce,
                *incremental,
                *pdf,
            ),
            Self::Benchmark {
                path,
                providers,
                replicates,
                parallel,
                minimal,
                incremental,
                timeout,
                stop_on_valid,
                count_unordered,
            } => main_handler::benchmark(
                path,
                providers,
                *replicates,
                *parallel,
                *minimal,
                *incremental,
                timeout.map(Duration::from_secs).unwrap_or(Duration::MAX),
                *stop_on_valid,
                *count_unordered,
            ),
            Self::BenchmarkStats { path } => {
                main_handler::benchmark_stats(path)
            }
            Self::GenerateSolutions {
                path,
                count,
                parallel,
            } => main_handler::generate_solutions(path, *count, *parallel),
            Self::Convert {
                path,
                format,
                randomize,
                reduce,
            } => main_handler::convert(path, format, *randomize, *reduce),
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

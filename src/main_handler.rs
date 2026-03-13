//! # Top-Level Functions
//!
//! Top-level functions to interact with AONav functinality

use crate::ao_adapters;
use crate::benchmark;
use crate::drivers::{self, Driver};
use crate::menu;
use crate::partition_navigation as pn;
use crate::util;

use ansi_term::Color::*;
use aograph as ao;
use indexmap::IndexMap;
use instant::Duration;
use jsongraph as jgf;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use strum::EnumString;

/// Valid input formats for conversion
#[derive(Debug, Clone, EnumString)]
pub enum ConversionInputFormat {
    EGraphSerialize,
    AOJsonGraph,
    Argus,
    Egglog,
}

/// Solutions to benchmark problems
#[derive(Debug, Serialize, Deserialize)]
pub struct ChosenSolutions {
    pub nonminimal_solutions: Vec<IndexMap<String, pn::Class>>,
    pub minimal_solutions: Vec<IndexMap<String, pn::Class>>,
}

fn load_chosen_solutions(path: &PathBuf) -> ChosenSolutions {
    let json_string = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&json_string).unwrap()
}

fn load_ao(path: &PathBuf) -> ao::Graph {
    let json_string = std::fs::read_to_string(path).unwrap();
    let jgf_data: jgf::Data = serde_json::from_str(&json_string).unwrap();

    let graph = match jgf_data {
        jgf::Data::Single { graph } => graph,
        jgf::Data::Multi { .. } => panic!("multi not supported"),
    };

    graph.try_into().unwrap()
}

/// Run interactively in the CLI
pub fn interact(
    graph_path: &PathBuf,
    providers: &Vec<menu::Provider>,
    reduce: bool,
    incremental_if_possible: bool,
    pdf: bool,
) -> Result<(), String> {
    let mut graph = load_ao(graph_path);

    if reduce {
        ao::algo::reduce(&mut graph);
    }

    let msg1 = format!(
        "Set of provable OR nodes: {}",
        ao::algo::provable_or_nodes(&graph).show(&graph)
    );

    println!("\n    {}", Yellow.bold().paint(msg1));

    let msg2 = format!("Goal is: {}", graph.or_at(graph.goal()));

    println!("    {}\n", Yellow.bold().paint(msg2));

    let start = pn::Exp::new(graph);
    let optional_start = if incremental_if_possible {
        Some(&start)
    } else {
        None
    };

    let provider = pbn::CompoundProvider::new(
        providers
            .iter()
            .map(|p| p.provider(optional_start))
            .collect(),
    );

    let checker = pn::oracle::Sufficient::new();

    let controller = pbn::Controller::new(
        util::Timer::infinite(),
        provider,
        checker,
        start,
        true,
    );

    let mut driver = drivers::Cli::new("sufficient".to_owned(), pdf);
    let _ = driver.drive(controller);

    Ok(())
}

/// Run a benchmark suite
pub fn benchmark(
    suite_path: &PathBuf,
    providers: &Vec<menu::Provider>,
    replicates: usize,
    parallel: bool,
    minimal: bool,
    incremental_if_possible: bool,
    timeout: Duration,
    stop_on_valid: bool,
    count_unordered: bool,
) -> Result<(), String> {
    if !suite_path.exists() {
        panic!("Path '{}' does not exist", suite_path.display())
    }

    let mut suite: Vec<benchmark::Problem> = vec![];

    for path in glob::glob(suite_path.join("*.json").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
    {
        let path_noext = path.with_extension("");

        if path_noext.extension().and_then(|e| e.to_str()) == Some("solutions")
        {
            continue;
        }

        let graph = load_ao(&path);

        let cs =
            load_chosen_solutions(&path_noext.with_extension("solutions.json"));

        let field = if minimal {
            cs.minimal_solutions
        } else {
            cs.nonminimal_solutions
        };

        suite.push(benchmark::Problem {
            name: path_noext.file_name().unwrap().to_str().unwrap().to_owned(),
            chosen_solutions: field
                .into_iter()
                .map(|labels| {
                    pn::Exp::from_labels(graph.clone(), labels).unwrap()
                })
                .collect(),
        });
    }

    let config = benchmark::Config {
        replicates,
        timeout,
        parallel,
        providers: providers.clone(),
        incremental_if_possible,
        stop_on_valid,
        count_unordered,
    };

    let runner = benchmark::Runner::new(config, std::io::stdout());
    runner.suite(&suite);

    Ok(())
}

#[derive(Serialize)]
struct BenchmarkStat {
    pub name: String,
    pub depth: Option<usize>,
    pub consumer_count: String,
    pub provider_count: String,
    pub premise_count: String,
}

/// Emit graph statistics for a benchmark suite
pub fn benchmark_stats(suite_path: &PathBuf) -> Result<(), String> {
    if !suite_path.exists() {
        panic!("Path '{}' does not exist", suite_path.display())
    }

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b',')
        .from_writer(Box::new(std::io::stdout()));

    let mut paths = glob::glob(suite_path.join("*.json").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    paths.sort();

    for path in paths {
        let path_noext = path.with_extension("");

        if path_noext.extension().and_then(|e| e.to_str()) == Some("solutions")
        {
            continue;
        }

        let graph = load_ao(&path);
        let oidxs = graph.or_indexes().collect::<Vec<_>>();
        let aidxs = graph.and_indexes().collect::<Vec<_>>();

        let s = BenchmarkStat {
            name: path_noext.file_name().unwrap().to_str().unwrap().to_owned(),
            consumer_count: oidxs
                .iter()
                .map(|o| graph.consumers(*o).count().to_string())
                .collect::<Vec<_>>()
                .join(";"),
            provider_count: oidxs
                .iter()
                .map(|o| graph.providers(*o).count().to_string())
                .collect::<Vec<_>>()
                .join(";"),
            premise_count: aidxs
                .iter()
                .map(|a| graph.premises(*a).count().to_string())
                .collect::<Vec<_>>()
                .join(";"),
            depth: graph.depth(),
        };

        wtr.serialize(s).unwrap();
        wtr.flush().unwrap();
    }

    Ok(())
}

fn generate_random_exp(graph: &ao::Graph) -> pn::Exp {
    let controller = pbn::Controller::new(
        util::Timer::infinite(),
        pn::providers::Random::new(pn::oracle::OptInc::NonIncremental),
        pn::oracle::Valid::new(pn::oracle::OptInc::NonIncremental),
        pn::Exp::new(graph.clone()),
        false,
    );

    let mut driver = drivers::Random::new(true);
    driver.drive(controller).unwrap()
}

fn generate_solutions_helper(
    path: PathBuf,
    solution_count: usize,
    parallel: bool,
) {
    let path_noext = path.with_extension("");

    if path_noext.extension().and_then(|e| e.to_str()) == Some("solutions") {
        return;
    }

    let graph = load_ao(&path);

    let nonminimal_solutions: Vec<_> = if parallel {
        (0..solution_count)
            .into_par_iter()
            .map(|_| generate_random_exp(&graph))
            .collect()
    } else {
        (0..solution_count)
            .into_iter()
            .map(|_| generate_random_exp(&graph))
            .collect()
    };

    let minimal_solutions: Vec<_> = if parallel {
        nonminimal_solutions
            .par_iter()
            .map(|e| pn::generate::assumption_minimized(e))
            .collect()
    } else {
        nonminimal_solutions
            .iter()
            .map(|e| pn::generate::assumption_minimized(e))
            .collect()
    };

    let cs = ChosenSolutions {
        nonminimal_solutions: nonminimal_solutions
            .into_iter()
            .map(|e| e.make_labels())
            .collect(),
        minimal_solutions: minimal_solutions
            .into_iter()
            .map(|e| e.make_labels())
            .collect(),
    };

    let mut file =
        File::create(path_noext.with_extension("solutions.json")).unwrap();
    writeln!(file, "{}", serde_json::to_string_pretty(&cs).unwrap()).unwrap();
}

/// Generate solutions for a benchmark suite
pub fn generate_solutions(
    suite_path: &PathBuf,
    solution_count: usize,
    parallel: bool,
) -> Result<(), String> {
    let paths: Vec<_> = glob::glob(suite_path.join("*.json").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    if parallel {
        paths.into_par_iter().for_each(|p| {
            generate_solutions_helper(p, solution_count, parallel)
        });
    } else {
        paths.into_iter().for_each(|p| {
            generate_solutions_helper(p, solution_count, parallel)
        });
    }
    Ok(())
}

/// Convert various representations to the AND/OR JSON Graph Format
pub fn convert(
    path: &PathBuf,
    format: &ConversionInputFormat,
    randomize: bool,
    reduce: bool,
) -> Result<(), String> {
    let mut ao: ao::Graph = match format {
        ConversionInputFormat::EGraphSerialize => {
            let es_egraph =
                egraph_serialize::EGraph::from_json_file(path).unwrap();
            ao_adapters::es_egraph_to_ao(&es_egraph)
        }
        ConversionInputFormat::AOJsonGraph => load_ao(path),
        ConversionInputFormat::Argus => {
            todo!()
        }
        ConversionInputFormat::Egglog => {
            let input = std::fs::read_to_string(path).unwrap();
            let mut egraph = egglog::EGraph::default();
            let egglog_program =
                egraph.parser.get_program_from_string(None, &input).unwrap();
            ao_adapters::try_from_egglog(egglog_program)?
        }
    };
    if reduce {
        ao::algo::reduce(&mut ao);
    }
    let mut jgf = jgf::Data::Single {
        graph: ao.try_into()?,
    };
    if randomize {
        let id_map = util::jgf_randomize_node_ids(&mut jgf);
        match &mut jgf {
            jgf::Data::Single { graph } => match &mut graph.metadata {
                Some(m) => m.insert(
                    "goal".to_owned(),
                    serde_json::Value::String(
                        id_map
                            .get(
                                m.get("goal")
                                    .expect("Need goal node for randomization")
                                    .as_str()
                                    .expect("Goal node not string for randomization"),
                            )
                            .expect("Goal node not found for randomization")
                            .clone(),
                    ),
                ),
                None => todo!(),
            },
            jgf::Data::Multi { .. } => {
                panic!("Randomization not supported for multi-graphs")
            },
        };
    }
    println!("{}", serde_json::to_string_pretty(&jgf).unwrap());
    Ok(())
}

///  Render an AND/OR graph in the AND/OR JSON Graph Format (stored in out/RENDERED.dot and out/RENDERED.pdf)
pub fn render(path: &PathBuf) -> Result<(), String> {
    let outdir = Path::new("out/");

    let ao = load_ao(path);

    let dot_path = outdir.join("RENDERED.dot");
    let mut dot_file = File::create(dot_path.clone()).unwrap();
    write!(&mut dot_file, "{}", ao.dot(&HashMap::new())).unwrap();

    let pdf_file = File::create(outdir.join("RENDERED.pdf")).unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg(dot_path)
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();

    Ok(())
}

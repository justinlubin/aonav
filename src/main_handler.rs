use crate::ao;
use crate::benchmark;
use crate::drivers::{self, Driver};
use crate::jgf;
use crate::menu;
use crate::partition_navigation;
use crate::pbn;
use crate::util::Timer;

use ansi_term::Color::*;
use indexmap::IndexSet;
use instant::Duration;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConversionInputFormat {
    EGraphSerialize,
    AOJsonGraph,
    Argus,
    Legacy,
    Egglog,
}

impl std::str::FromStr for ConversionInputFormat {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChosenSolutions {
    pub chosen_solutions: Vec<IndexSet<ao::NodeId>>,
}

fn load_chosen_solutions(path: &PathBuf) -> Option<Vec<IndexSet<ao::NodeId>>> {
    let json_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let cs: ChosenSolutions = serde_json::from_str(&json_string).unwrap();
    Some(cs.chosen_solutions)
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

pub fn interact(
    graph_path: &PathBuf,
    providers: &Vec<menu::Provider>,
) -> Result<(), String> {
    let graph = load_ao(graph_path);

    let msg1 = format!(
        "Set of provable OR nodes: {}",
        ao::algo::provable_or_nodes(&graph).show(&graph)
    );

    println!("\n    {}", Yellow.bold().paint(msg1));

    let msg2 = format!("Goal is: {}", graph.or_at(graph.goal()));

    println!("    {}\n", Yellow.bold().paint(msg2));

    let provider = pbn::CompoundProvider::new(
        providers.iter().map(|p| p.provider()).collect(),
    );
    let checker = partition_navigation::Valid::new();

    let controller = pbn::Controller::new(
        Timer::infinite(),
        provider,
        checker,
        partition_navigation::Exp::new(graph),
    );

    let mut driver = drivers::CliDriver::new();
    let _ = driver.drive(controller);

    Ok(())
}

pub fn benchmark(suite_path: &PathBuf) -> Result<(), String> {
    if !suite_path.exists() {
        panic!("Path '{}' does not exist", suite_path.display())
    }

    let mut suite: Vec<benchmark::BenchmarkEntry> = vec![];

    for path in glob::glob(suite_path.join("*.json").to_str().unwrap())
        .unwrap()
        .filter_map(Result::ok)
    {
        let path_noext = path.with_extension("");

        if path_noext.extension().and_then(|e| e.to_str()) == Some("solutions")
        {
            continue;
        }

        suite.push(benchmark::BenchmarkEntry {
            name: path_noext.file_name().unwrap().to_str().unwrap().to_owned(),
            graph: load_ao(&path),
            chosen_solutions: load_chosen_solutions(
                &path_noext.with_extension("solutions.json"),
            ),
        });
    }

    let config = benchmark::Config {
        replicates: 3,
        timeout: Duration::from_secs(1),
        parallel: false,
    };

    let runner = benchmark::Runner::new(config, std::io::stdout());
    runner.suite(&suite);

    Ok(())
}

pub fn generate_solutions(suite_path: &PathBuf) -> Result<(), String> {
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

        let cs = ChosenSolutions {
            chosen_solutions: ao::algo::proper_axiom_sets(&graph, graph.goal())
                .into_iter()
                .map(|axs| axs.ids(&graph))
                .collect(),
        };

        let mut file =
            File::create(path_noext.with_extension("solutions.json")).unwrap();
        writeln!(file, "{}", serde_json::to_string_pretty(&cs).unwrap())
            .unwrap();
    }

    Ok(())
}

pub fn convert(
    path: &PathBuf,
    format: &ConversionInputFormat,
) -> Result<(), String> {
    match format {
        ConversionInputFormat::EGraphSerialize => {
            let es_egraph =
                egraph_serialize::EGraph::from_json_file(path).unwrap();
            let ao = ao::convert::es_egraph_to_ao(&es_egraph);
            let jgf = jgf::Data::Single {
                graph: ao.try_into()?,
            };
            println!("{}", serde_json::to_string_pretty(&jgf).unwrap());
            Ok(())
        }
        ConversionInputFormat::AOJsonGraph => {
            let ao = load_ao(path);
            let jgf = jgf::Data::Single {
                graph: ao.try_into()?,
            };
            println!("{}", serde_json::to_string_pretty(&jgf).unwrap());
            Ok(())
        }
        ConversionInputFormat::Argus => {
            ao::convert::argus_to_and_or(path);
            Ok(())
        }
        ConversionInputFormat::Legacy => {
            let mut lines =
                crate::util::read_lines(&format!("{}", path.display()))
                    .unwrap();
            let goal = lines.remove(0).trim().to_owned();
            let proof_system = crate::legacy::proof_system(&lines);
            let ao = crate::legacy::to_ao(proof_system, goal);
            let jgf = jgf::Data::Single {
                graph: ao.try_into()?,
            };
            println!("{}", serde_json::to_string_pretty(&jgf).unwrap());
            Ok(())
        }
        ConversionInputFormat::Egglog => {
            let input = std::fs::read_to_string(path).unwrap();
            let mut egraph = egglog::EGraph::default();
            let egglog_program =
                egraph.parser.get_program_from_string(None, &input).unwrap();
            let ao: ao::Graph = egglog_program.try_into()?;
            let jgf = jgf::Data::Single {
                graph: ao.try_into()?,
            };
            println!("{}", serde_json::to_string_pretty(&jgf).unwrap());
            Ok(())
        }
    }
}

pub fn render(path: &PathBuf) -> Result<(), String> {
    let outdir = Path::new("out/");

    let ao = load_ao(path);

    let dot_path = outdir.join("RENDERED.dot");
    let mut dot_file = File::create(dot_path.clone()).unwrap();
    write!(&mut dot_file, "{}", ao.dot(&IndexSet::new())).unwrap();

    let pdf_file = File::create(outdir.join("RENDERED.pdf")).unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg(dot_path)
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();

    Ok(())
}

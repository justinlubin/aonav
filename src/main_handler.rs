use crate::ao;
use crate::ao_navigation;
use crate::benchmark;
use crate::convert;
use crate::drivers::{self, Driver};
use crate::jgf;
use crate::pbn;
use crate::util::Timer;

use ansi_term::Color::*;
use indexmap::{IndexMap, IndexSet};
use instant::Duration;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConversionInputFormat {
    EGraphSerialize,
    AOJsonGraph,
    Argus,
}

impl std::str::FromStr for ConversionInputFormat {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChosenSolutions {
    pub chosen_solutions: Vec<Vec<String>>,
}

fn load_chosen_solutions(
    path: &PathBuf,
) -> Option<Vec<ao_navigation::AxiomSet>> {
    let json_string = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let cs: ChosenSolutions = serde_json::from_str(&json_string).unwrap();
    Some(
        cs.chosen_solutions
            .into_iter()
            .map(ao_navigation::AxiomSet::from_vec)
            .collect(),
    )
}

fn load_ao(path: &PathBuf) -> ao::Graph<(), ()> {
    let json_string = std::fs::read_to_string(path).unwrap();
    let jgf_data: jgf::Data = serde_json::from_str(&json_string).unwrap();

    let graph = match jgf_data {
        jgf::Data::Single { graph } => graph,
        jgf::Data::Multi { .. } => panic!("multi not supported"),
    };

    graph.try_into().unwrap()
}

fn emit_graph<A, O>(graph: &ao::Graph<A, O>, name: &str) {
    let mut dot_file = File::create(format!("out/{}.dot", name)).unwrap();
    write!(&mut dot_file, "{}", graph.dot(&IndexSet::new())).unwrap();

    let pdf_file = File::create(format!("out/{}.pdf", name)).unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg(format!("out/{}.dot", name))
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();
}

pub fn interact(graph_path: &PathBuf) -> Result<(), String> {
    let graph = load_ao(graph_path);
    emit_graph(&graph, "initial");

    let mut reduced = graph.clone();
    reduced.reduce();
    emit_graph(&reduced, "reduced");

    let msg1 = format!(
        "Set of provable OR nodes: {:?}",
        graph
            .provable_or_nodes()
            .iter()
            .map(|oid| graph.or_label(*oid))
            .collect::<Vec<_>>()
    );

    println!("\n    {}", Yellow.bold().paint(msg1));

    let msg2 = format!("Goal is: {}", graph.or_label(graph.goal_oid()));

    println!("    {}\n", Yellow.bold().paint(msg2));

    let provider = ao_navigation::GreedyProvider::new(graph.clone());
    let checker = ao_navigation::GoalProvable::new(graph.clone());

    let controller = pbn::Controller::new(
        Timer::infinite(),
        provider,
        checker,
        ao_navigation::AxiomSet::empty(),
    );

    let mut driver = drivers::CliDriver;
    let _ = driver.drive(controller);

    Ok(())
}

pub fn benchmark(suite_path: &PathBuf) -> Result<(), String> {
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

pub fn convert(
    path: &PathBuf,
    format: &ConversionInputFormat,
) -> Result<(), String> {
    match format {
        ConversionInputFormat::EGraphSerialize => {
            let es_egraph =
                egraph_serialize::EGraph::from_json_file(path).unwrap();
            let ao = convert::es_egraph_to_ao(&es_egraph);
            let jgf_graph: jgf::Graph = ao.into();
            println!("{}", serde_json::to_string_pretty(&jgf_graph).unwrap());
            Ok(())
        }
        ConversionInputFormat::AOJsonGraph => {
            let ao = load_ao(path);
            let jgf_graph: jgf::Graph = ao.into();
            println!("{}", serde_json::to_string_pretty(&jgf_graph).unwrap());
            Ok(())
        }
        ConversionInputFormat::Argus => {
            todo!()
        }
    }
}

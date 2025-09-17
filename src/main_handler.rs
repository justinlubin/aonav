use crate::ao;
use crate::ao_navigation;
use crate::convert;
use crate::jgf;
use crate::pbn;
use crate::util::Timer;

use ansi_term::Color::*;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConversionInputFormat {
    EGraphSerialize,
    AOJsonGraph,
}

impl std::str::FromStr for ConversionInputFormat {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
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

fn emit_graph<A, O>(graph: &ao::Graph<A, O>) {
    let mut dot_file = File::create("out/out.dot").unwrap();
    write!(&mut dot_file, "{}", graph.dot(&IndexSet::new())).unwrap();

    let pdf_file = File::create("out/out.pdf").unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg("out/out.dot")
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();
}

pub fn interact(graph_path: &PathBuf) -> Result<(), String> {
    let graph = load_ao(graph_path);
    emit_graph(&graph);

    let provider = ao_navigation::IncorrectProvider { graph };

    let checker = ao_navigation::TargetReachableChecker::new("GOAL".to_owned());

    let mut controller = pbn::Controller::new(
        Timer::infinite(),
        provider,
        checker,
        ao_navigation::AxiomSet::new(),
    );

    let mut round = 0;

    loop {
        round += 1;

        let valid = controller.valid();
        let mut options = controller.provide().unwrap();

        if !valid && options.is_empty() {
            println!("{}", Red.bold().paint("Not possible!"));
            return Ok(());
        }

        println!(
            "{}\n\n{}\n\n    {}\n\n{}\n",
            Fixed(8).paint(format!("══ Round {} {}", round, "═".repeat(40))),
            Cyan.bold().paint("Working expression:"),
            controller.working_expression(),
            Cyan.bold().paint("Possible next steps:"),
        );

        for (i, option) in options.iter().cloned().enumerate() {
            print!("  {}) ", i + 1);
            match option {
                ao_navigation::AOStep::Add(s) => {
                    println!("{}", Yellow.paint(format!("+ {}", s)))
                }
            }
        }

        if valid {
            println!(
                "  f) Expression is {}, finish navigation",
                Green.bold().paint("valid")
            )
        }

        let idx = loop {
            print!(
                "\n{} {}\n\n> ",
                Purple.bold().paint("Which step would you like to take?"),
                Fixed(8).paint("('q' to quit)"),
            );
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input == "q" {
                return Ok(());
            }

            if valid && input == "f" {
                break None;
            }

            match input.parse::<usize>() {
                Ok(choice) => {
                    if 1 <= choice && choice <= options.len() {
                        break Some(choice - 1);
                    } else {
                        continue;
                    }
                }
                Err(_) => continue,
            };
        };

        let idx = match idx {
            Some(idx) => idx,
            None => break,
        };

        controller.decide(options.swap_remove(idx))
    }

    let output = format!("{}", controller.working_expression());

    println!(
        "\n{}\n\n    {}",
        Green.bold().paint("Final expression:"),
        output
    );

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
    }
}

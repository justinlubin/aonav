use crate::ao;
use crate::drivers::{self, Driver};
use crate::partition_navigation;
use crate::pbn;
use crate::util::Timer;

use indexmap::IndexSet;
use instant::{Duration, Instant};
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct BenchmarkEntry {
    pub name: String,
    pub graph: ao::Graph,
    pub chosen_solutions: Option<Vec<IndexSet<ao::NodeId>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    // Key
    pub strategy: String,
    pub name: String,
    pub chosen_solution: usize,
    pub replicate: usize,

    // Values
    pub completed: bool,
    pub success: bool,
    pub duration: u128,
    pub decisions: usize,
}

/// Benchmark configuration
pub struct Config {
    /// How many times to run each entry
    pub replicates: usize,
    /// When to cut off the benchmark early
    pub timeout: Duration,
    /// Whether or not to run the benchmarks in parallel
    pub parallel: bool,
}

/// The core data structure for running benchmarks
pub struct Runner {
    config: Config,
    wtr: Arc<Mutex<csv::Writer<Box<dyn io::Write + Send + 'static>>>>,
}

impl Runner {
    /// Create a new benchmark runner. The `writer` argument is the location
    /// that the benchmark results will get written to (e.g., stdout).
    pub fn new(
        config: Config,
        writer: impl io::Write + Send + 'static,
    ) -> Self {
        Self {
            wtr: Arc::new(Mutex::new(
                csv::WriterBuilder::new()
                    .delimiter(b',')
                    .from_writer(Box::new(writer)),
            )),
            config,
        }
    }

    fn entry(&self, entry: &BenchmarkEntry) {
        let chosen_solutions = match &entry.chosen_solutions {
            Some(cs) => cs,
            None => return,
        };

        for (i, solution) in chosen_solutions.iter().enumerate() {
            for replicate in 0..self.config.replicates {
                let now = Instant::now();

                let provider =
                    partition_navigation::providers::Remaining::new();
                let checker = partition_navigation::oracle::Valid::new();
                let controller = pbn::Controller::new(
                    Timer::finite(self.config.timeout),
                    provider,
                    checker,
                    partition_navigation::Exp::new(entry.graph.clone()),
                    false,
                );
                let mut driver = drivers::RandomizedSolutionDrivenDriver::new(
                    solution.clone(),
                );
                let e = driver.drive(controller);

                let duration = now.elapsed().as_millis();

                let (completed, success) = match e {
                    None => (false, false),
                    Some(e) => (
                        true,
                        e.filter_class(|c| {
                            c == partition_navigation::Class::True {
                                force_use: true,
                                assume: Some(true),
                            }
                        })
                        .ids(e.graph())
                            == *solution,
                    ),
                };

                let r = BenchmarkResult {
                    strategy: "naive-pbn".to_owned(),
                    name: entry.name.clone(),
                    chosen_solution: i,
                    replicate,
                    completed,
                    success,
                    duration,
                    decisions: driver.decisions(),
                };

                let wtr = Arc::clone(&self.wtr);
                let mut wtr = wtr.lock().unwrap();
                wtr.serialize(r).unwrap();
                wtr.flush().unwrap();
            }
        }
    }

    /// Run a benchmark suite
    pub fn suite(&self, entries: &Vec<BenchmarkEntry>) {
        if self.config.parallel {
            entries.into_par_iter().for_each(|e| self.entry(e));
        } else {
            entries.into_iter().for_each(|e| self.entry(e));
        }
    }
}

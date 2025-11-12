use crate::drivers::{self, Driver};
use crate::partition_navigation as pn;
use crate::pbn;
use crate::util::Timer;

use instant::{Duration, Instant};
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct BenchmarkEntry {
    pub name: String,
    // Contain a graph and a selected partition
    pub chosen_solutions: Vec<pn::Exp>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    // Key
    pub provider: String,
    pub name: String,
    pub chosen_solution: usize,
    pub replicate: usize,

    // Values
    pub success: bool,
    pub duration: u128,
    pub total_decisions: usize,
    pub unique_decisions: usize,
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
        for (i, solution) in entry.chosen_solutions.iter().enumerate() {
            for replicate in 0..self.config.replicates {
                let now = Instant::now();

                let provider = pn::providers::Remaining::new();
                let checker = pn::oracle::Valid::new();
                let controller = pbn::Controller::new(
                    Timer::finite(self.config.timeout),
                    provider,
                    checker,
                    pn::Exp::new(solution.graph().clone()),
                    false,
                );
                let mut driver = drivers::SolutionDriven::new(solution.clone());
                let _ = driver.drive(controller);

                let duration = now.elapsed().as_millis();

                let r = BenchmarkResult {
                    provider: "Remaining".to_owned(),
                    name: entry.name.clone(),
                    chosen_solution: i,
                    replicate,
                    success: true,
                    duration,
                    total_decisions: driver.total_decisions(),
                    unique_decisions: driver.unique_decisions(),
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

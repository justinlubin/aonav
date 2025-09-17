use crate::ao;
use crate::ao_navigation;

use instant::{Duration, Instant};
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct BenchmarkEntry {
    pub name: String,
    pub graph: ao::Graph<(), ()>,
    pub chosen_solutions: Option<Vec<ao_navigation::AxiomSet>>,
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
        let now = Instant::now();

        todo!();

        let r = BenchmarkResult {
            strategy: todo!(),
            name: entry.name.clone(),
            chosen_solution: todo!(),
            replicate: todo!(),
            completed: todo!(),
            success: todo!(),
            duration: todo!(),
            decisions: todo!(),
        };

        let wtr = Arc::clone(&self.wtr);
        let mut wtr = wtr.lock().unwrap();
        wtr.serialize(r).unwrap();
        wtr.flush().unwrap();
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

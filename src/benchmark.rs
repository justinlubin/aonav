//! # Benchmarking
//!
//! This module defines everything necessary to benchmark the step providers in
//! this project.

use crate::drivers::{self, Driver};
use crate::menu;
use crate::partition_navigation as pn;
use crate::util::Timer;

use indicatif::{ParallelProgressIterator, ProgressIterator};
use instant::{Duration, Instant};
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::sync::{Arc, Mutex};

/// Describes name and solution of a single benchmark problem
#[derive(Debug)]
pub struct Problem {
    pub name: String,
    // Contains a graph and a selected partition
    pub chosen_solutions: Vec<pn::Exp>,
}

/// Description of a single benchmark problem
#[derive(Debug)]
pub struct BenchmarkEntry {
    pub provider: menu::Provider,
    pub name: String,
    pub chosen_solution: usize,
    pub replicate: usize,
}

/// Data for a single benchmark problem
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    // Key
    pub provider: menu::Provider,
    pub name: String,
    pub chosen_solution: usize,
    pub replicate: usize,

    // Values
    pub success: bool,
    pub duration: u128,
    pub total_decisions: usize,
    pub unique_decisions: usize,
    pub latencies: String,
}

/// Benchmark configuration
pub struct Config {
    /// How many times to run each entry
    pub replicates: usize,
    /// When to cut off the benchmark early
    pub timeout: Duration,
    /// Whether or not to run the benchmarks in parallel
    pub parallel: bool,
    /// The step providers to use
    pub providers: Vec<menu::Provider>,
    /// Whether or not to use incrementality (if possible)
    pub incremental_if_possible: bool,
    /// Whether or not to stop on validity (or just sufficiency)
    pub stop_on_valid: bool,
    /// Count decisions as cardinality of options presented (unordered output)
    pub count_unordered: bool,
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

    fn entry(&self, entry: BenchmarkEntry, solution: pn::Exp) {
        let now = Instant::now();

        let start = pn::Exp::new(solution.graph().clone());
        let optional_start = if self.config.incremental_if_possible {
            Some(&start)
        } else {
            None
        };

        let controller = if self.config.stop_on_valid {
            pbn::Controller::new(
                Timer::finite(self.config.timeout),
                pbn::CompoundProvider::new(vec![entry
                    .provider
                    .provider(optional_start)]),
                pn::oracle::Valid::new(pn::oracle::OptInc::NonIncremental),
                start,
                false,
            )
        } else {
            pbn::Controller::new(
                Timer::finite(self.config.timeout),
                pbn::CompoundProvider::new(vec![entry
                    .provider
                    .provider(optional_start)]),
                pn::oracle::Sufficient::new(),
                start,
                false,
            )
        };

        let mut driver =
            drivers::SolutionDriven::new(solution, self.config.count_unordered);
        let success = driver.drive(controller).is_some();

        let duration = now.elapsed().as_millis();

        let r = BenchmarkResult {
            provider: entry.provider,
            name: entry.name,
            chosen_solution: entry.chosen_solution,
            replicate: entry.replicate,
            success,
            duration,
            total_decisions: driver.total_decisions(),
            unique_decisions: driver.unique_decisions(),
            latencies: driver
                .latencies()
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(";"),
        };

        let wtr = Arc::clone(&self.wtr);
        let mut wtr = wtr.lock().unwrap();
        wtr.serialize(r).unwrap();
        wtr.flush().unwrap();
    }

    /// Run a benchmark suite
    pub fn suite(&self, problems: &Vec<Problem>) {
        if self.config.parallel {
            problems
                .into_par_iter()
                .progress_count(problems.len() as u64)
                .for_each(|problem| {
                    self.config.providers.par_iter().for_each(|provider| {
                        problem
                            .chosen_solutions
                            .par_iter()
                            .enumerate()
                            .for_each(|(chosen_solution, solution)| {
                                (0..self.config.replicates)
                                    .into_par_iter()
                                    .for_each(|replicate| {
                                        self.entry(
                                            BenchmarkEntry {
                                                provider: *provider,
                                                name: problem.name.clone(),
                                                chosen_solution,
                                                replicate,
                                            },
                                            solution.clone(),
                                        );
                                    });
                            })
                    })
                });
        } else {
            problems
                .into_iter()
                .progress_count(problems.len() as u64)
                .for_each(|problem| {
                    self.config.providers.iter().for_each(|provider| {
                        problem.chosen_solutions.iter().enumerate().for_each(
                            |(chosen_solution, solution)| {
                                (0..self.config.replicates)
                                    .into_iter()
                                    .for_each(|replicate| {
                                        self.entry(
                                            BenchmarkEntry {
                                                provider: *provider,
                                                name: problem.name.clone(),
                                                chosen_solution,
                                                replicate,
                                            },
                                            solution.clone(),
                                        );
                                    });
                            },
                        )
                    })
                });
        }
    }
}

//! # Utilities

use std::collections::HashMap;

use indexmap::IndexMap;
use instant::Duration;
use instant::Instant;
use jsongraph as jgf;
use pbn::Timer as _;
use rand::distr::{Alphabetic, SampleString};

////////////////////////////////////////////////////////////////////////////////
// Early cutoff

/// The type of reasons that a computation may have been cut off early.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EarlyCutoff {
    TimerExpired,
}

impl std::fmt::Display for EarlyCutoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EarlyCutoff::TimerExpired => write!(f, "TimerExpired"),
        }
    }
}

impl std::error::Error for EarlyCutoff {}

////////////////////////////////////////////////////////////////////////////////
// Timer

#[derive(Debug, Clone)]
enum TimerInner {
    Finite { end: Instant },
    Infinite,
}

/// The type of timers; these can be used to cut off a computation early based
/// on a timeout. These are used cooperatively, and [`Timer::tick`] must be
/// called frequently enough so that there is a chance to interrupt the
/// computation.
#[derive(Debug)]
pub struct Timer(TimerInner);

impl Timer {
    /// A finite-duration timer.
    pub fn finite(duration: Duration) -> Self {
        if duration == Duration::MAX {
            Timer(TimerInner::Infinite)
        } else {
            Timer(TimerInner::Finite {
                end: Instant::now() + duration,
            })
        }
    }

    /// An infinite-duration timer (will never cut off the computation).
    pub fn infinite() -> Self {
        Timer(TimerInner::Infinite)
    }
}

impl pbn::Timer for Timer {
    type EarlyCutoff = EarlyCutoff;

    /// Tick the timer (cooperatively check to see if the computation needs to
    /// stop).
    fn tick(&self) -> Result<(), EarlyCutoff> {
        match self.0 {
            TimerInner::Finite { end } => {
                if Instant::now() > end {
                    Err(EarlyCutoff::TimerExpired)
                } else {
                    Ok(())
                }
            }
            TimerInner::Infinite => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Small utilities

/// Take the cartesion product of a set of chocies
pub fn cartesian_product<K: Clone + Eq + std::hash::Hash, V: Clone>(
    timer: &Timer,
    choices: IndexMap<K, Vec<V>>,
) -> Result<Vec<IndexMap<K, V>>, EarlyCutoff> {
    let mut results = vec![IndexMap::new()];
    for (k, vs) in choices.iter() {
        let mut new_results = vec![];
        for map in results {
            timer.tick()?;
            for v in vs {
                let mut new_map = map.clone();
                new_map.insert(k.clone(), v.clone());
                new_results.push(new_map)
            }
        }
        results = new_results;
    }
    Ok(results)
}

////////////////////////////////////////////////////////////////////////////////
// JSON Graph Format helpers

/// Generate random nodes ids for benchmrking
pub fn jgf_randomize_node_ids(data: &mut jgf::Data) -> HashMap<String, String> {
    match data {
        jgf::Data::Single { graph } => jgf_graph_randomize_node_ids(graph),
        jgf::Data::Multi { .. } => {
            panic!("Randomize not supported for multi-graphs")
        }
    }
}

fn jgf_graph_randomize_node_ids(
    graph: &mut jgf::Graph,
) -> HashMap<String, String> {
    let nodes = match graph.nodes.take() {
        Some(ns) => ns,
        None => return HashMap::new(),
    };
    let mut id_map = HashMap::new();
    let mut new_nodes = IndexMap::new();
    for (old_id, node) in nodes {
        let new_id = loop {
            let candidate = Alphabetic.sample_string(&mut rand::rng(), 32);
            if !id_map.contains_key(&candidate) {
                break candidate;
            }
        };
        id_map.insert(old_id.clone(), new_id.clone());
        let _ = new_nodes.insert(new_id, node);
    }
    graph.nodes = Some(new_nodes);

    let edges = match graph.edges.take() {
        Some(es) => es,
        None => return id_map,
    };

    let mut new_edges = vec![];

    for mut edge in edges {
        edge.source = id_map
            .get(&edge.source)
            .expect(&format!("Invalid source id '{}'", edge.source))
            .clone();
        edge.target = id_map
            .get(&edge.target)
            .expect(&format!("Invalid target id '{}'", edge.target))
            .clone();
        new_edges.push(edge)
    }

    graph.edges = Some(new_edges);

    id_map
}

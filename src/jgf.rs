//! Types for JSON Graph Format (JGF) v2
//!
//! Schema available at:
//!     https://jsongraphformat.info/v2.0/json-graph-schema.json

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use rand::distr::{Alphabetic, SampleString};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    Single {
        graph: Graph,
    },
    Multi {
        #[serde(skip_serializing_if = "Option::is_none")]
        graphs: Option<Vec<Graph>>,
    },
}

impl Data {
    pub fn randomize_node_ids(&mut self) -> HashMap<String, String> {
        match self {
            Data::Single { graph } => graph.randomize_node_ids(),
            Data::Multi { .. } => {
                panic!("Randomize not supported for multi-graphs")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "default_true")]
    pub directed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub graph_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<IndexMap<String, Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<Vec<Edge>>,
}

impl Graph {
    pub fn randomize_node_ids(&mut self) -> HashMap<String, String> {
        let nodes = match self.nodes.take() {
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
        self.nodes = Some(new_nodes);

        let edges = match self.edges.take() {
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

        self.edges = Some(new_edges);

        id_map
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    #[serde(skip_serializing_if = "is_true")]
    #[serde(default = "default_true")]
    pub directed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
}

fn default_true() -> bool {
    true
}

fn is_true(b: &bool) -> bool {
    *b
}

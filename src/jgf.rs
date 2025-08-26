//! Types for JSON Graph Format (JGF) v2
//!
//! Schema available at:
//!     https://jsongraphformat.info/v2.0/json-graph-schema.json

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    Single { graph: Graph },
    Multi { graphs: Option<Vec<Graph>> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
    pub id: Option<String>,
    pub label: Option<String>,
    #[serde(default = "default_true")]
    pub directed: bool,
    #[serde(rename = "type")]
    pub graph_type: Option<String>,
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
    pub nodes: Option<IndexMap<String, Node>>,
    pub edges: Option<Vec<Edge>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub label: Option<String>,
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    pub relation: Option<String>,
    #[serde(default = "default_true")]
    pub directed: bool,
    pub label: Option<String>,
    pub metadata: Option<IndexMap<String, serde_json::Value>>,
}

fn default_true() -> bool {
    true
}

//! Types for JSON Graph Format (JGF) v2
//!
//! Schema available at:
//!     https://jsongraphformat.info/v2.0/json-graph-schema.json

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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

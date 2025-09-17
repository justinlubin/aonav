use crate::jgf;

use indexmap::{IndexMap, IndexSet};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

pub enum AONode<A, O> {
    And(A),
    Or(O),
}

impl std::fmt::Display for AONode<String, String> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::And(x) => x,
            Self::Or(x) => x,
        };
        write!(f, "{}", s)
    }
}

pub struct AOEdge;

impl std::fmt::Display for AOEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "")
    }
}

pub struct AndOrGraph<A, O>(petgraph::graph::Graph<AONode<A, O>, AOEdge>);

impl TryFrom<jgf::Graph> for AndOrGraph<String, String> {
    type Error = String;

    fn try_from(value: jgf::Graph) -> Result<Self, Self::Error> {
        let nodes = value.nodes.ok_or("missing nodes")?;
        let edges = value.edges.ok_or("missing edges")?;

        let mut petgraph_ids = HashMap::with_capacity(nodes.len());

        let mut ret =
            petgraph::graph::Graph::with_capacity(nodes.len(), edges.len());

        for (node_id, node_val) in nodes {
            let metadata = node_val
                .metadata
                .ok_or(format!("missing metadata for '{}'", node_id))?;
            let n = match metadata
                .get("kind")
                .ok_or(format!("missing 'kind' metadata for '{}'", node_id))?
                .as_str()
                .ok_or(format!(
                    "'kind' metadata for '{}' not a string",
                    node_id
                ))?
                .to_ascii_uppercase()
                .as_str()
            {
                "AND" => AONode::And(node_id.clone()),
                "OR" => AONode::Or(node_id.clone()),
                _ => {
                    return Err(format!(
                        "unknown 'kind' metadata for '{}'",
                        node_id
                    ))
                }
            };
            let pid = ret.add_node(n);
            let _ = petgraph_ids.insert(node_id, pid);
        }

        for edge in edges {
            let source_pid = *petgraph_ids.get(&edge.source).unwrap();
            let target_pid = *petgraph_ids.get(&edge.target).unwrap();
            let _ = ret.add_edge(source_pid, target_pid, AOEdge);
        }

        Ok(AndOrGraph(ret))
    }
}

impl From<AndOrGraph<String, String>> for jgf::Graph {
    fn from(ao: AndOrGraph<String, String>) -> Self {
        jgf::Graph {
            id: None,
            label: None,
            directed: true,
            graph_type: None,
            metadata: None,
            nodes: Some(
                ao.0.node_weights()
                    .map(|nw| match nw {
                        AONode::And(key) => (
                            key.clone(),
                            jgf::Node {
                                label: None,
                                metadata: Some(IndexMap::from([(
                                    "kind".to_owned(),
                                    serde_json::Value::String("AND".to_owned()),
                                )])),
                            },
                        ),
                        AONode::Or(key) => (
                            key.clone(),
                            jgf::Node {
                                label: None,
                                metadata: Some(IndexMap::from([(
                                    "kind".to_owned(),
                                    serde_json::Value::String("OR".to_owned()),
                                )])),
                            },
                        ),
                    })
                    .collect(),
            ),
            edges: Some(
                ao.0.edge_references()
                    .map(|e| jgf::Edge {
                        id: None,
                        source: format!("{}", ao.0[e.source()]),
                        target: format!("{}", ao.0[e.target()]),
                        relation: None,
                        directed: true,
                        label: None,
                        metadata: None,
                    })
                    .collect(),
            ),
        }
    }
}

impl<A: Clone, O: Clone> AndOrGraph<A, O> {
    pub fn or_nodes(&self) -> Vec<O> {
        let mut ret = vec![];
        for nw in self.0.node_weights() {
            match nw {
                AONode::And(_) => continue,
                AONode::Or(o) => ret.push(o.clone()),
            }
        }
        ret
    }
}

impl AndOrGraph<String, String> {
    fn node_format(
        highlighted_nodes: &IndexSet<String>,
        n: &AONode<String, String>,
    ) -> String {
        let (key, base) = match n {
            AONode::Or(k) => (
                k,
                "color=darkslateblue,fontcolor=darkslateblue,penwidth=2"
                    .to_string(),
            ),
            AONode::And(k) => (
                k,
                "shape=rectangle,color=gray35,fontcolor=gray35,margin=0"
                    .to_string(),
            ),
        };
        base + if highlighted_nodes.contains(key) {
            ",fillcolor=yellow"
        } else {
            ""
        }
    }

    pub fn dot(&self, highlighted_nodes: &IndexSet<String>) -> String {
        let get_node_attrs =
            |_, (_, n)| Self::node_format(highlighted_nodes, n);

        let d = petgraph::dot::Dot::with_attr_getters(
            &self.0,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|g, e| match g.node_weight(e.target()) {
                Some(AONode::And(_)) => "color=red".to_string(),
                Some(AONode::Or(_)) => "color=blue, style=dashed".to_string(),
                _ => panic!("malformatted graph"),
            },
            &get_node_attrs,
        );
        format!("{}", d)
    }
}

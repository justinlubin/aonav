use crate::jgf;

use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::io::Write;

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

impl AndOrGraph<String, String> {
    pub fn dot(&self) -> String {
        let d = petgraph::dot::Dot::with_attr_getters(
            &self.0,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|g, e| match g.node_weight(e.target()) {
                Some(AONode::And(_)) => "color=red".to_string(),
                Some(AONode::Or(_)) => "color=blue, style=dashed".to_string(),
                _ => panic!("malformatted graph"),
            },
            &|_, (_, n)| match n {
                AONode::Or(_) => {
                    "color=darkslateblue, fontcolor=darkslateblue, penwidth=2"
                        .to_string()
                }
                AONode::And(_) => {
                    "shape=rectangle, color=gray35, fontcolor=gray35, margin=0"
                        .to_string()
                }
            },
        );
        return format!("{}", d);
    }
}

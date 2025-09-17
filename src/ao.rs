use crate::jgf;

use indexmap::{IndexMap, IndexSet};
use petgraph::graph as pg;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;
use std::fmt;

////////////////////////////////////////////////////////////////////////////////
// Nodes

pub enum Node<A, O> {
    And(String, Option<A>),
    Or(String, Option<O>),
}

impl<A, O> Node<A, O> {
    pub fn label(&self) -> &str {
        match self {
            Node::And(label, _) => label,
            Node::Or(label, _) => label,
        }
    }

    pub fn is_and(&self) -> bool {
        match self {
            Node::And(_, _) => true,
            Node::Or(_, _) => false,
        }
    }

    pub fn is_or(&self) -> bool {
        match self {
            Node::And(_, _) => false,
            Node::Or(_, _) => true,
        }
    }
}

impl<A, O> fmt::Display for Node<A, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Edges

pub struct Edge;

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

////////////////////////////////////////////////////////////////////////////////
// Graph construction, conversion, and formatting

pub struct NodeId(pg::NodeIndex);

pub struct Graph<A, O> {
    pg: pg::Graph<Node<A, O>, Edge>,
}

impl<A, O> TryFrom<jgf::Graph> for Graph<A, O> {
    type Error = String;

    fn try_from(value: jgf::Graph) -> Result<Self, Self::Error> {
        let nodes = value.nodes.ok_or("missing nodes")?;
        let edges = value.edges.ok_or("missing edges")?;

        let mut petgraph_ids = HashMap::with_capacity(nodes.len());

        let mut ret = pg::Graph::with_capacity(nodes.len(), edges.len());

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
                "AND" => Node::And(node_id.clone(), None),
                "OR" => Node::Or(node_id.clone(), None),
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
            let _ = ret.add_edge(source_pid, target_pid, Edge);
        }

        Ok(Graph { pg: ret })
    }
}

impl<A, O> From<Graph<A, O>> for jgf::Graph {
    fn from(ao: Graph<A, O>) -> Self {
        jgf::Graph {
            id: None,
            label: None,
            directed: true,
            graph_type: None,
            metadata: None,
            nodes: Some(
                ao.pg
                    .node_weights()
                    .map(|nw| match nw {
                        Node::And(label, _) => (
                            label.clone(),
                            jgf::Node {
                                label: None,
                                metadata: Some(IndexMap::from([(
                                    "kind".to_owned(),
                                    serde_json::Value::String("AND".to_owned()),
                                )])),
                            },
                        ),
                        Node::Or(label, _) => (
                            label.clone(),
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
                ao.pg
                    .edge_references()
                    .map(|e| jgf::Edge {
                        id: None,
                        source: format!("{}", ao.pg[e.source()]),
                        target: format!("{}", ao.pg[e.target()]),
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

impl<A, O> Graph<A, O> {
    fn node_format(
        highlighted_nodes: &IndexSet<String>,
        n: &Node<A, O>,
    ) -> String {
        let (label, base) = match n {
            Node::Or(label, _) => (
                label,
                "color=darkslateblue,fontcolor=darkslateblue,penwidth=2"
                    .to_string(),
            ),
            Node::And(label, _) => (
                label,
                "shape=rectangle,color=gray35,fontcolor=gray35,margin=0"
                    .to_string(),
            ),
        };
        base + if highlighted_nodes.contains(label) {
            ",fillcolor=yellow"
        } else {
            ""
        }
    }

    pub fn dot(&self, highlighted_nodes: &IndexSet<String>) -> String {
        let get_node_attrs =
            |_, (_, n)| Self::node_format(highlighted_nodes, n);

        let d = petgraph::dot::Dot::with_attr_getters(
            &self.pg,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|g, e| match g.node_weight(e.target()) {
                Some(Node::And(_, _)) => "color=red".to_string(),
                Some(Node::Or(_, _)) => "color=blue, style=dashed".to_string(),
                _ => panic!("malformatted graph"),
            },
            &get_node_attrs,
        );
        format!("{}", d)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Graph operations

impl<A, O> Graph<A, O> {
    pub fn label(&self, nid: NodeId) -> &str {
        self.pg[nid.0].label()
    }

    pub fn or_nodes(&self) -> impl Iterator<Item = NodeId> + use<'_, A, O> {
        self.pg.node_indices().filter_map(|nid| {
            if self.pg[nid].is_or() {
                Some(NodeId(nid))
            } else {
                None
            }
        })
    }

    pub fn sources(&self) -> impl Iterator<Item = NodeId> + use<'_, A, O> {
        self.pg.externals(Direction::Incoming).filter_map(|nid| {
            if self.pg[nid].is_and() {
                Some(NodeId(nid))
            } else {
                None
            }
        })
    }

    pub fn make_axiom(&mut self, nid: NodeId) {
        let label = &self.pg[nid.0];
        let ax_nid = self.pg.add_node(Node::And(format!("ax:{}", label), None));
        let _ = self.pg.add_edge(ax_nid, nid.0, Edge);
    }

    pub fn remove_axiom(&mut self, nid: NodeId) {
        for er in self.pg.edges_directed(nid.0, Direction::Incoming) {
            let source_nid = er.source();
            if self.pg[source_nid].label().starts_with("ax:") {
                let _ = self.pg.remove_node(source_nid);
                return;
            }
        }
        panic!("'{}' is not an axiom", self.pg[nid.0].label())
    }

    pub fn preds(
        &self,
        nid: NodeId,
    ) -> impl Iterator<Item = NodeId> + use<'_, A, O> {
        self.pg
            .edges_directed(nid.0, Direction::Incoming)
            .map(|er| NodeId(er.source()))
    }

    pub fn succs(
        &self,
        nid: NodeId,
    ) -> impl Iterator<Item = NodeId> + use<'_, A, O> {
        self.pg
            .edges_directed(nid.0, Direction::Outgoing)
            .map(|er| NodeId(er.target()))
    }
}

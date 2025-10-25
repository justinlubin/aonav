use indexmap::{IndexMap, IndexSet};
use petgraph::stable_graph as pg;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use petgraph::Direction;
use std::fmt;

////////////////////////////////////////////////////////////////////////////////
// Nodes

pub type NodeId = String;

#[allow(non_camel_case_types)]
pub type nodeid = str;

#[derive(Debug, Clone)]
pub enum Node<A, O> {
    And {
        id: NodeId,
        label: Option<String>,
        data: Option<A>,
    },
    Or {
        id: NodeId,
        label: Option<String>,
        data: Option<O>,
    },
}

impl<A, O> Node<A, O> {
    pub fn id(&self) -> &nodeid {
        match self {
            Node::And { id, .. } | Node::Or { id, .. } => id,
        }
    }

    pub fn label(&self) -> &Option<String> {
        match self {
            Node::And { label, .. } | Node::Or { label, .. } => label,
        }
    }

    pub fn is_and(&self) -> bool {
        match self {
            Node::And { .. } => true,
            Node::Or { .. } => false,
        }
    }

    pub fn is_or(&self) -> bool {
        match self {
            Node::And { .. } => false,
            Node::Or { .. } => true,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Node::And { .. } => "AND",
            Node::Or { .. } => "OR",
        }
    }
}

impl<A, O> fmt::Display for Node<A, O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.label() {
            Some(s) => write!(f, "[{}]\n{}", self.id(), s),
            None => write!(f, "{}", self.id()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Edges

#[derive(Debug, Clone)]
struct Edge;

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

////////////////////////////////////////////////////////////////////////////////
// Graphs

#[derive(Debug, Clone)]
pub struct Graph<A, O> {
    pg: pg::StableGraph<Node<A, O>, Edge>,
    goal: pg::NodeIndex,
}

pub type Generic = serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AIdx(pg::NodeIndex);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OIdx(pg::NodeIndex);

impl<A, O> Graph<A, O> {
    // Creation

    pub fn new<'a>(
        nodes: impl Iterator<Item = Node<A, O>>,
        edges: impl Iterator<Item = (NodeId, NodeId)>,
        goal: &'a nodeid,
    ) -> Result<Self, String> {
        let mut pg = pg::StableGraph::new();

        let mut translation = IndexMap::new();

        for node in nodes {
            let nid = node.id().to_owned();
            if translation.contains_key(&nid) {
                return Err(format!("Duplicate node '{}'", nid));
            }
            let pid = pg.add_node(node);
            translation.insert(nid, pid);
        }

        for (source_nid, target_nid) in edges {
            let source_pid = *translation
                .get(&source_nid)
                .ok_or(format!("Source '{}' is not a node", source_nid))?;

            let target_pid = *translation
                .get(&target_nid)
                .ok_or(format!("Target '{}' is not a node", target_nid))?;

            match (&pg[source_pid], &pg[target_pid]) {
                (Node::And { .. }, Node::And { .. }) => {
                    return Err(format!(
                        "Cannot connect AND node '{}' to AND node '{}'",
                        source_nid, target_nid
                    ))
                }
                (Node::Or { .. }, Node::Or { .. }) => {
                    return Err(format!(
                        "Cannot connect Or node '{}' to Or node '{}'",
                        source_nid, target_nid
                    ))
                }
                (Node::And { .. }, Node::Or { .. })
                | (Node::Or { .. }, Node::And { .. }) => (),
            };

            let _ = pg.add_edge(source_pid, target_pid, Edge);
        }

        let goal = match translation.get(goal).map(|pid| (pid, &pg[*pid])) {
            Some((pid, Node::Or { .. })) => *pid,
            _ => return Err(format!("Goal node '{}' is not an OR node", goal)),
        };

        for pid in pg.node_indices() {
            let node = &pg[pid];
            if !node.is_and() {
                continue;
            }
            let mut it = pg.edges_directed(pid, Direction::Incoming);
            let _ = it.next();
            if it.next().is_some() {
                return Err(format!(
                    "AND node '{}' has more than one conclusion",
                    node.id()
                ));
            }
        }

        Ok(Self { pg, goal })
    }

    pub fn map<F, G, A2, O2>(
        &self,
        mut or_map: F,
        mut and_map: G,
    ) -> Graph<A2, O2>
    where
        F: FnMut(Option<&O>) -> Option<O2>,
        G: FnMut(Option<&A>) -> Option<A2>,
    {
        Graph {
            pg: self.pg.map(
                |_, n| match n {
                    Node::And { id, label, data } => Node::And {
                        id: id.clone(),
                        label: label.clone(),
                        data: and_map(data.as_ref()),
                    },
                    Node::Or { id, label, data } => Node::Or {
                        id: id.clone(),
                        label: label.clone(),
                        data: or_map(data.as_ref()),
                    },
                },
                |_, e| e.clone(),
            ),
            goal: self.goal,
        }
    }

    // Basics

    pub fn or_indexes(&self) -> impl Iterator<Item = OIdx> + '_ {
        self.pg.node_indices().filter_map(|pid| {
            if self.pg[pid].is_or() {
                Some(OIdx(pid))
            } else {
                None
            }
        })
    }

    pub fn and_indexes(&self) -> impl Iterator<Item = AIdx> + '_ {
        self.pg.node_indices().filter_map(|pid| {
            if self.pg[pid].is_and() {
                Some(AIdx(pid))
            } else {
                None
            }
        })
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node<A, O>> {
        self.pg.node_weights()
    }

    pub fn edges(&self) -> impl Iterator<Item = (&Node<A, O>, &Node<A, O>)> {
        self.pg
            .edge_references()
            .map(|e| (&self.pg[e.source()], &self.pg[e.target()]))
    }

    pub fn goal(&self) -> OIdx {
        OIdx(self.goal)
    }

    pub fn sources(&self) -> impl Iterator<Item = AIdx> + '_ {
        self.pg.externals(Direction::Outgoing).filter_map(|pid| {
            let node = &self.pg[pid];
            if node.is_and() {
                Some(AIdx(pid))
            } else {
                None
            }
        })
    }

    // Indexing

    pub fn or_at(&self, o: OIdx) -> &Node<A, O> {
        &self.pg[o.0]
    }

    pub fn or_data_ref(&self, o: OIdx) -> Option<&O> {
        match &self.pg[o.0] {
            Node::Or { data, .. } => data.as_ref(),
            _ => panic!("OR-index is not valid"),
        }
    }

    pub fn or_data_mut(&mut self, o: OIdx) -> Option<&mut O> {
        match &mut self.pg[o.0] {
            Node::Or { data, .. } => data.as_mut(),
            _ => panic!("OR-index is not valid"),
        }
    }

    pub fn premises(&self, a: AIdx) -> impl Iterator<Item = OIdx> + '_ {
        self.pg
            .edges_directed(a.0, Direction::Outgoing)
            .map(|er| OIdx(er.source()))
    }

    pub fn conclusion(&self, a: AIdx) -> OIdx {
        OIdx(
            self.pg
                .edges_directed(a.0, Direction::Incoming)
                .next()
                .unwrap()
                .source(),
        )
    }

    pub fn providers(&self, o: OIdx) -> impl Iterator<Item = AIdx> + '_ {
        self.pg
            .edges_directed(o.0, Direction::Outgoing)
            .map(|er| AIdx(er.source()))
    }

    pub fn consumers(&self, o: OIdx) -> impl Iterator<Item = AIdx> + '_ {
        self.pg
            .edges_directed(o.0, Direction::Incoming)
            .map(|er| AIdx(er.source()))
    }

    pub fn provider_cone(&self, o: OIdx) -> IndexSet<OIdx> {
        self.providers(o)
            .flat_map(|a| self.premises(a))
            .flat_map(|o| std::iter::once(o).chain(self.provider_cone(o)))
            .collect()
    }

    // Modifications

    pub fn set_goal(&mut self, o: OIdx) {
        self.goal = o.0;
    }

    pub fn make_axiom(&mut self, o: OIdx) {
        let ax_pid = self.pg.add_node(Node::And {
            id: format!("ax:{}", self.pg[o.0].id()),
            label: None,
            data: None,
        });
        let _ = self.pg.add_edge(o.0, ax_pid, Edge);
    }

    pub fn make_axioms(&mut self, oidxs: impl Iterator<Item = OIdx>) {
        for o in oidxs {
            self.make_axiom(o)
        }
    }

    // DOT formatting

    fn node_format(
        highlighted_nodes: &IndexSet<OIdx>,
        pid: pg::NodeIndex,
        node: &Node<A, O>,
    ) -> String {
        let base = match node {
            Node::Or { .. } => {
                "color=darkslateblue,fontcolor=darkslateblue,penwidth=2"
                    .to_string()
            }
            Node::And { .. } => {
                "shape=rectangle,color=gray35,fontcolor=gray35,margin=0"
                    .to_string()
            }
        };
        base + if highlighted_nodes.contains(&OIdx(pid)) {
            ",style=filled,fillcolor=yellow"
        } else {
            ""
        }
    }

    pub fn dot(&self, highlighted_nodes: &IndexSet<OIdx>) -> String {
        let get_node_attrs =
            |_, (pid, node)| Self::node_format(highlighted_nodes, pid, node);

        let d = petgraph::dot::Dot::with_attr_getters(
            &self.pg,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|g, e| match g[e.source()] {
                Node::And { .. } => "color=red".to_string(),
                Node::Or { .. } => "color=blue, style=dashed".to_string(),
            },
            &get_node_attrs,
        );
        format!("{}", d)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Node sets

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSet {
    pub set: IndexSet<OIdx>,
}

impl NodeSet {
    pub fn ids<A, O>(&self, graph: &Graph<A, O>) -> IndexSet<NodeId> {
        self.set
            .iter()
            .map(|oid| graph.or_at(*oid).id().to_owned())
            .collect()
    }

    pub fn show<A, O>(&self, graph: &Graph<A, O>) -> String {
        if self.set.is_empty() {
            "∅".to_owned()
        } else {
            let mut first = true;
            let mut s = "".to_owned();
            for oid in &self.set {
                let ax = graph.or_at(*oid);
                s += &format!("{}{}", if first { "{" } else { ", " }, ax);
                first = false;
            }
            s + "}"
        }
    }
}

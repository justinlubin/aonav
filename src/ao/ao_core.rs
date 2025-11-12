use indexmap::{IndexMap, IndexSet};
use petgraph::stable_graph as pg;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use petgraph::Direction;
use std::collections::HashMap;
use std::fmt;

////////////////////////////////////////////////////////////////////////////////
// Nodes

pub type NodeId = String;

#[allow(non_camel_case_types)]
pub type nodeid = str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    And,
    Or,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    id: NodeId,
    label: Option<String>,
    kind: NodeKind,
}

impl Node {
    pub fn new(id: NodeId, label: Option<String>, kind: NodeKind) -> Self {
        Self { id, label, kind }
    }

    pub fn id(&self) -> &nodeid {
        &self.id
    }

    pub fn label(&self) -> Option<&nodeid> {
        self.label.as_ref().map(|x| x.as_str())
    }

    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    pub fn is_and(&self) -> bool {
        self.kind == NodeKind::And
    }

    pub fn is_or(&self) -> bool {
        self.kind == NodeKind::Or
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.label() {
            Some(s) => write!(f, "{}", s),
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
pub struct Graph {
    pg: pg::StableGraph<Node, Edge>,
    goal: pg::NodeIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AIdx(pg::NodeIndex);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OIdx(pg::NodeIndex);

impl Graph {
    // Creation

    pub fn new(
        nodes: impl Iterator<Item = Node>,
        edges: impl Iterator<Item = (NodeId, NodeId)>,
        goal: &nodeid,
    ) -> Result<Self, String> {
        let mut pg = pg::StableGraph::new();

        let mut translation = IndexMap::new();

        for node in nodes {
            let nid = node.id.to_owned();
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

            match (&pg[source_pid].kind, &pg[target_pid].kind) {
                (NodeKind::And, NodeKind::And) => {
                    return Err(format!(
                        "Cannot connect AND node '{}' to AND node '{}'",
                        source_nid, target_nid
                    ))
                }
                (NodeKind::Or, NodeKind::Or) => {
                    return Err(format!(
                        "Cannot connect Or node '{}' to Or node '{}'",
                        source_nid, target_nid
                    ))
                }
                (NodeKind::And, NodeKind::Or)
                | (NodeKind::Or, NodeKind::And) => (),
            };

            let _ = pg.add_edge(source_pid, target_pid, Edge);
        }

        let goal = match translation.get(goal).map(|pid| (pid, pg[*pid].kind)) {
            Some((pid, NodeKind::Or)) => *pid,
            _ => return Err(format!("Goal node '{}' is not an OR node", goal)),
        };

        for pid in pg.node_indices() {
            let node = &pg[pid];
            if !node.is_and() {
                continue;
            }
            let mut it = pg.edges_directed(pid, Direction::Incoming);
            if !it.next().is_some() {
                return Err(format!(
                    "AND node '{}' has no conclusion",
                    node.id
                ));
            }
            if it.next().is_some() {
                return Err(format!(
                    "AND node '{}' has more than one conclusion",
                    node.id
                ));
            }
        }

        Ok(Self { pg, goal })
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

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.pg.node_weights()
    }

    pub fn edges(&self) -> impl Iterator<Item = (&Node, &Node)> {
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

    pub fn or_leaves(&self) -> impl Iterator<Item = OIdx> + '_ {
        self.pg.externals(Direction::Outgoing).filter_map(|pid| {
            let node = &self.pg[pid];
            if node.is_or() {
                Some(OIdx(pid))
            } else {
                None
            }
        })
    }

    pub fn find_or_by_id(&self, id: &nodeid) -> Option<OIdx> {
        self.pg.node_indices().find_map(|pid| {
            let n = &self.pg[pid];
            if n.is_or() && n.id() == id {
                Some(OIdx(pid))
            } else {
                None
            }
        })
    }

    // Indexing

    pub fn or_at(&self, o: OIdx) -> &Node {
        &self.pg[o.0]
    }

    pub fn and_at(&self, a: AIdx) -> &Node {
        &self.pg[a.0]
    }

    pub fn premises(&self, a: AIdx) -> impl Iterator<Item = OIdx> + '_ {
        self.pg
            .edges_directed(a.0, Direction::Outgoing)
            .map(|er| OIdx(er.target()))
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
            .map(|er| AIdx(er.target()))
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
        let ax_pid = self.pg.add_node(Node {
            id: format!("ax:{}", self.pg[o.0].id),
            label: None,
            kind: NodeKind::And,
        });
        let _ = self.pg.add_edge(o.0, ax_pid, Edge);
    }

    pub fn make_axioms(&mut self, oidxs: impl Iterator<Item = OIdx>) {
        for o in oidxs {
            self.make_axiom(o)
        }
    }

    pub fn or_remove(&mut self, oidx: OIdx) {
        for aidx in self.providers(oidx).collect::<Vec<_>>() {
            self.pg.remove_node(aidx.0);
        }
        self.pg.remove_node(oidx.0);
    }

    // DOT formatting

    fn node_format(
        highlights: &HashMap<OIdx, String>,
        pid: pg::NodeIndex,
        node: &Node,
    ) -> String {
        let base = match node.kind {
            NodeKind::Or => {
                "color=darkslateblue,fontcolor=darkslateblue,penwidth=2"
                    .to_string()
            }
            NodeKind::And => {
                "shape=rectangle,color=gray35,fontcolor=gray35,margin=0"
                    .to_string()
            }
        };
        base + &match highlights.get(&OIdx(pid)) {
            Some(c) => format!(",style=filled,fillcolor={}", c),
            None => "".to_string(),
        }
    }

    pub fn dot(&self, highlights: &HashMap<OIdx, String>) -> String {
        let get_node_attrs =
            |_, (pid, node)| Self::node_format(highlights, pid, node);

        let d = petgraph::dot::Dot::with_attr_getters(
            &self.pg,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|g, e| match g[e.source()].kind {
                NodeKind::And => "color=red".to_string(),
                NodeKind::Or => "color=blue, style=dashed".to_string(),
            },
            &get_node_attrs,
        );
        format!("{}", d)
    }
}

////////////////////////////////////////////////////////////////////////////////
// AND sets and OR sets

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndSet {
    pub set: IndexSet<AIdx>,
}

impl AndSet {
    pub fn new() -> Self {
        AndSet {
            set: IndexSet::new(),
        }
    }

    pub fn singleton(aidx: AIdx) -> Self {
        AndSet {
            set: IndexSet::from([aidx]),
        }
    }

    pub fn ids(&self, graph: &Graph) -> IndexSet<NodeId> {
        self.set
            .iter()
            .map(|aidx| graph.and_at(*aidx).id.to_owned())
            .collect()
    }

    pub fn show(&self, graph: &Graph) -> String {
        if self.set.is_empty() {
            "∅".to_owned()
        } else {
            let mut first = true;
            let mut s = "".to_owned();
            for aidx in &self.set {
                let ax = graph.and_at(*aidx);
                s += &format!("{}{}", if first { "{" } else { ", " }, ax);
                first = false;
            }
            s + "}"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrSet {
    pub set: IndexSet<OIdx>,
}

impl OrSet {
    pub fn new() -> Self {
        OrSet {
            set: IndexSet::new(),
        }
    }

    pub fn singleton(oidx: OIdx) -> Self {
        OrSet {
            set: IndexSet::from([oidx]),
        }
    }

    pub fn ids(&self, graph: &Graph) -> IndexSet<NodeId> {
        self.set
            .iter()
            .map(|oidx| graph.or_at(*oidx).id.to_owned())
            .collect()
    }

    pub fn show(&self, graph: &Graph) -> String {
        if self.set.is_empty() {
            "∅".to_owned()
        } else {
            let mut first = true;
            let mut s = "".to_owned();
            for oidx in &self.set {
                let ax = graph.or_at(*oidx);
                s += &format!("{}{}", if first { "{" } else { ", " }, ax);
                first = false;
            }
            s + "}"
        }
    }
}

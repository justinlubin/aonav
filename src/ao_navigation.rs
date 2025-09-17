use crate::ao;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;

////////////////////////////////////////////////////////////////////////////////
// Basics

// Expressions

#[derive(Debug, Clone)]
pub struct AxiomSet(IndexSet<ao::NodeLabel>);

impl AxiomSet {
    pub fn new() -> Self {
        AxiomSet(IndexSet::new())
    }
}

impl std::fmt::Display for AxiomSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "∅")
        } else {
            let mut first = true;
            for ax in &self.0 {
                write!(f, "{}{}", if first { "{" } else { ", " }, ax)?;
                first = false;
            }
            write!(f, "}}")
        }
    }
}

// Steps

#[derive(Debug, Clone)]
pub enum AOStep {
    Add(ao::NodeLabel),
}

impl pbn::Step for AOStep {
    type Exp = AxiomSet;

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            AOStep::Add(s) => {
                let mut ret = e.clone();
                if ret.0.insert(s.clone()) {
                    Some(ret)
                } else {
                    None
                }
            }
        }
    }
}

// Checker

pub struct GoalProvable<A, O> {
    graph: ao::Graph<A, O>,
}

impl<A, O> GoalProvable<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self { graph }
    }
}

impl<A: Clone, O: Clone> pbn::ValidityChecker for GoalProvable<A, O> {
    type Exp = AxiomSet;

    // TODO switch to using backward reasoning
    fn check(&self, e: &Self::Exp) -> bool {
        let mut graph_with_axioms = self.graph.clone();
        for node_label in &e.0 {
            graph_with_axioms.make_axiom(graph_with_axioms.find_oid(node_label))
        }
        graph_with_axioms
            .provable_or_nodes()
            .contains(&graph_with_axioms.goal_oid())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Providers

pub struct IncorrectProvider<A, O> {
    pub graph: ao::Graph<A, O>,
}

impl<A, O> pbn::StepProvider for IncorrectProvider<A, O> {
    type Step = AOStep;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &AxiomSet,
    ) -> Result<Vec<AOStep>, EarlyCutoff> {
        let mut steps = vec![];

        for node in self.graph.or_nodes() {
            let label = self.graph.or_label(node).to_owned();
            if e.0.contains(&label) {
                continue;
            }
            steps.push(AOStep::Add(label))
        }

        Ok(steps)
    }
}

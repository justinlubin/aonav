use crate::ao;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;

////////////////////////////////////////////////////////////////////////////////
// Basics

// Expressions

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxiomSet(IndexSet<ao::NodeLabel>);

impl AxiomSet {
    pub fn empty() -> Self {
        AxiomSet(IndexSet::new())
    }

    pub fn from_vec(set: Vec<ao::NodeLabel>) -> Self {
        AxiomSet(set.into_iter().collect())
    }

    pub fn from_slice<const N: usize>(set: [ao::NodeLabel; N]) -> Self {
        AxiomSet(IndexSet::from(set))
    }

    pub fn to_vec(&self) -> Vec<ao::NodeLabel> {
        self.0.iter().cloned().collect()
    }

    pub fn contains(&self, label: &ao::NodeLabel) -> bool {
        self.0.contains(label)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl std::fmt::Display for AOStep {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AOStep::Add(s) => {
                write!(f, "+ {}", s)
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

    fn check(&self, e: &Self::Exp) -> bool {
        let mut ax_graph = self.graph.clone();
        ax_graph.make_axioms(e.0.iter());
        ax_graph.provable_or_node(ax_graph.goal_oid())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Greedy provider

pub struct GreedyProvider {
    proper_axiom_sets: Vec<AxiomSet>,
}

pub fn proper_axiom_sets<A: Clone, O: Clone>(
    graph: &ao::Graph<A, O>,
) -> Vec<AxiomSet> {
    let mut ret: Vec<AxiomSet> = vec![];
    let mut agenda: Vec<AxiomSet> = vec![AxiomSet::empty()];
    let or_labels: Vec<&str> = graph.or_labels().collect();

    while let Some(axs) = agenda.pop() {
        let mut ax_graph = graph.clone();
        ax_graph.make_axioms(axs.0.iter());
        if ax_graph.provable_or_node(ax_graph.goal_oid()) {
            let mut new_ret = vec![];
            let mut should_add = true;
            for ret_axs in ret {
                if ret_axs.0.is_subset(&axs.0) {
                    should_add = false;
                }
                if !axs.0.is_subset(&ret_axs.0) {
                    new_ret.push(ret_axs);
                }
            }
            if should_add {
                new_ret.push(axs);
            }
            ret = new_ret;
        } else {
            'label: for label in or_labels.iter().cloned() {
                let mut new_axs = axs.clone();
                if new_axs.0.insert(label.to_owned()) {
                    for ret_axs in &ret {
                        if ret_axs.0.is_subset(&new_axs.0) {
                            continue 'label;
                        }
                    }
                    agenda.push(new_axs);
                };
            }
        }
    }

    if log::log_enabled!(log::Level::Debug) {
        let mut msg = "Proper axiom sets: [".to_owned();
        for ret_axs in &ret {
            msg += &format!(" {}", ret_axs);
        }
        msg += " ]";
        log::debug!("{}", msg);
    }

    ret
}

impl GreedyProvider {
    pub fn new<A: Clone, O: Clone>(graph: ao::Graph<A, O>) -> Self {
        Self {
            proper_axiom_sets: proper_axiom_sets(&graph),
        }
    }
}

impl pbn::StepProvider for GreedyProvider {
    type Step = AOStep;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &AxiomSet,
    ) -> Result<Vec<AOStep>, EarlyCutoff> {
        let mut next_labels = IndexSet::new();

        for axs in &self.proper_axiom_sets {
            if e.0.is_subset(&axs.0) {
                next_labels.extend(axs.0.difference(&e.0).cloned())
            }
        }

        let mut steps: Vec<_> =
            next_labels.into_iter().map(AOStep::Add).collect();
        steps.sort();

        Ok(steps)
    }
}

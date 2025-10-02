use crate::ao;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// Basics

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxiomSet {
    pub set: IndexSet<ao::OIdx>,
}

impl AxiomSet {
    pub fn ids<A, O>(&self, graph: &ao::Graph<A, O>) -> IndexSet<ao::NodeId> {
        self.set
            .iter()
            .map(|oid| graph.or_at(*oid).id().to_owned())
            .collect()
    }

    pub fn show<A, O>(&self, graph: &ao::Graph<A, O>) -> String {
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

// TODO implement sorting for axiom sets

#[derive(Debug, Clone)]
pub struct Exp<A, O> {
    graph: ao::Graph<A, O>,
    axioms: AxiomSet,
}

impl<A, O> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Exp {
            graph,
            axioms: AxiomSet {
                set: IndexSet::new(),
            },
        }
    }

    pub fn graph(&self) -> &ao::Graph<A, O> {
        &self.graph
    }

    pub fn axioms(&self) -> &AxiomSet {
        &self.axioms
    }
}

impl<A, O> std::fmt::Display for Exp<A, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.axioms.show(&self.graph))
    }
}

// Steps

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<A, O> {
    Add(ao::OIdx, PhantomData<(A, O)>),
    Refine(ao::OIdx, AxiomSet),
}

impl<A, O> Step<A, O> {
    pub fn show(&self, e: &Exp<A, O>) -> String {
        match self {
            Step::Add(oid, _) => {
                format!("+ {}", e.graph.or_at(*oid))
            }
            Step::Refine(oid, axs) => {
                format!("{} -> {}", e.graph.or_at(*oid), axs.show(&e.graph))
            }
        }
    }
}

impl<A: Clone, O: Clone> pbn::Step for Step<A, O> {
    type Exp = Exp<A, O>;

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Step::Add(s, _) => {
                let mut ret = e.clone();
                if ret.axioms.set.insert(s.clone()) {
                    Some(ret)
                } else {
                    None
                }
            }
            Step::Refine(x, axs) => {
                let mut ret = e.clone();
                if ret.axioms.set.swap_remove(x) {
                    ret.axioms.set.extend(axs.set.iter().cloned());
                    Some(ret)
                } else {
                    None
                }
            }
        }
    }
}

// TODO implement sorting for steps

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
    type Exp = Exp<A, O>;

    fn check(&self, e: &Self::Exp) -> bool {
        let mut ax_graph = self.graph.clone();
        ax_graph.make_axioms(e.axioms.set.iter().cloned());
        ax_graph.provable_or_node(ax_graph.goal())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Compound provider (composition of other providers)

pub struct CompoundProvider<S: pbn::Step> {
    providers: Vec<Box<dyn pbn::StepProvider<Step = S>>>,
}

impl<S: pbn::Step> CompoundProvider<S> {
    pub fn new(providers: Vec<Box<dyn pbn::StepProvider<Step = S>>>) -> Self {
        Self { providers }
    }
}

impl<S: pbn::Step> pbn::StepProvider for CompoundProvider<S> {
    type Step = S;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &<Self::Step as pbn::Step>::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];
        for p in &mut self.providers {
            steps.extend(p.provide(timer, e)?);
        }
        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Greedy add provider

pub struct GreedyAddProvider<A, O> {
    phantom: PhantomData<(A, O)>,
    proper_axiom_sets: Vec<AxiomSet>,
}

pub fn proper_axiom_sets<A: Clone, O: Clone>(
    graph: &ao::Graph<A, O>,
) -> Vec<AxiomSet> {
    let mut ret: Vec<AxiomSet> = vec![];
    let mut agenda: Vec<AxiomSet> = vec![AxiomSet {
        set: IndexSet::new(),
    }];
    let or_indexes: Vec<ao::OIdx> = graph.or_indexes().collect();

    while let Some(axs) = agenda.pop() {
        let mut ax_graph = graph.clone();
        ax_graph.make_axioms(axs.set.iter().cloned());
        if ax_graph.provable_or_node(ax_graph.goal()) {
            let mut new_ret = vec![];
            let mut should_add = true;
            for ret_axs in ret {
                if ret_axs.set.is_subset(&axs.set) {
                    should_add = false;
                }
                if !axs.set.is_subset(&ret_axs.set) {
                    new_ret.push(ret_axs);
                }
            }
            if should_add {
                new_ret.push(axs);
            }
            ret = new_ret;
        } else {
            'label: for o in or_indexes.iter().cloned() {
                let mut new_axs = axs.clone();
                if new_axs.set.insert(o) {
                    for ret_axs in &ret {
                        if ret_axs.set.is_subset(&new_axs.set) {
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
            msg += &format!(" {}", ret_axs.show(graph));
        }
        msg += " ]";
        log::debug!("{}", msg);
    }

    ret
}

impl<A: Clone, O: Clone> GreedyAddProvider<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self {
            phantom: PhantomData,
            proper_axiom_sets: proper_axiom_sets(&graph),
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for GreedyAddProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut next_labels = IndexSet::new();

        for axs in &self.proper_axiom_sets {
            if e.axioms.set.is_subset(&axs.set) {
                next_labels.extend(axs.set.difference(&e.axioms.set).cloned())
            }
        }

        let steps: Vec<_> = next_labels
            .into_iter()
            .map(|o| Step::Add(o, PhantomData))
            .collect();

        // TODO sort
        // steps.sort();

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Greedy refine provider

pub struct GreedyRefineProvider<A, O> {
    graph: ao::Graph<A, O>,
}

impl<A, O> GreedyRefineProvider<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self { graph }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for GreedyRefineProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];

        for x in &e.axioms.set {
            self.graph.set_goal(*x);
            for axs in proper_axiom_sets(&self.graph) {
                if axs.set == IndexSet::from([x.clone()]) {
                    continue;
                }
                steps.push(Step::Refine(*x, axs))
            }
        }

        Ok(steps)
    }
}

use crate::ao;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// Expressions

#[derive(Debug, Clone)]
pub struct Exp<A, O> {
    graph: ao::Graph<A, O>,
    axioms: ao::NodeSet,
}

impl<A, O> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Exp {
            graph,
            axioms: ao::NodeSet {
                set: IndexSet::new(),
            },
        }
    }

    pub fn graph(&self) -> &ao::Graph<A, O> {
        &self.graph
    }

    pub fn axioms(&self) -> &ao::NodeSet {
        &self.axioms
    }
}

impl<A, O> std::fmt::Display for Exp<A, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.axioms.show(&self.graph))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Steps

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<A, O> {
    Add(ao::OIdx, PhantomData<(A, O)>),
    Refine(ao::OIdx, ao::NodeSet),
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

////////////////////////////////////////////////////////////////////////////////
// Validity checker

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
        ao::algo::provable(&ax_graph, ax_graph.goal())
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

pub struct NaiveAddProvider<A, O> {
    phantom: PhantomData<(A, O)>,
    proper_axiom_sets: Vec<ao::NodeSet>,
}

impl<A: Clone, O: Clone> NaiveAddProvider<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self {
            phantom: PhantomData,
            proper_axiom_sets: ao::algo::proper_axiom_sets(&graph),
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for NaiveAddProvider<A, O> {
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

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Greedy refine provider

pub struct NaiveRefineProvider<A, O> {
    graph: ao::Graph<A, O>,
}

impl<A, O> NaiveRefineProvider<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self { graph }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for NaiveRefineProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];

        for x in &e.axioms.set {
            self.graph.set_goal(*x);
            for axs in ao::algo::proper_axiom_sets(&self.graph) {
                if axs.set == IndexSet::from([x.clone()]) {
                    continue;
                }
                steps.push(Step::Refine(*x, axs))
            }
        }

        Ok(steps)
    }
}

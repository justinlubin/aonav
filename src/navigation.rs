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
    committed: ao::NodeSet,
    allowed: ao::NodeSet,
    rejected: ao::NodeSet,
}

impl<A, O> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Exp {
            graph,
            committed: ao::NodeSet {
                set: IndexSet::new(),
            },
            allowed: ao::NodeSet {
                set: IndexSet::new(),
            },
            rejected: ao::NodeSet {
                set: IndexSet::new(),
            },
        }
    }

    pub fn graph(&self) -> &ao::Graph<A, O> {
        &self.graph
    }

    pub fn committed(&self) -> &ao::NodeSet {
        &self.committed
    }
}

impl<A, O> std::fmt::Display for Exp<A, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "c: {} a: {} r: {}",
            self.committed.show(&self.graph),
            self.allowed.show(&self.graph),
            self.rejected.show(&self.graph)
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// Steps

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<A, O> {
    /// Add node to allowed set
    Add(ao::OIdx, PhantomData<(A, O)>),

    /// Mark a node as permanently rejected
    Reject(ao::OIdx),

    /// Refine a node in the allowed set
    Refine(ao::OIdx, ao::NodeSet),

    /// Commit node from allowed set to committed set
    Commit(ao::OIdx),

    /// Sequence two steps
    Seq(Box<Step<A, O>>, Box<Step<A, O>>),
}

impl<A, O> Step<A, O> {
    pub fn sequence(
        mut steps: impl Iterator<Item = Step<A, O>>,
    ) -> Option<Step<A, O>> {
        let mut step = steps.next()?;

        for s in steps {
            step = Step::Seq(Box::new(step), Box::new(s));
        }

        Some(step)
    }

    pub fn show(&self, e: &Exp<A, O>) -> String {
        match self {
            Step::Add(oid, _) => {
                format!("+ {}", e.graph.or_at(*oid))
            }
            Step::Reject(oid) => {
                format!("- {}", e.graph.or_at(*oid))
            }
            Step::Refine(oid, axs) => {
                format!("{} -> {}", e.graph.or_at(*oid), axs.show(&e.graph))
            }
            Step::Commit(oid) => {
                format!("commit {}", e.graph.or_at(*oid))
            }
            Step::Seq(s1, s2) => {
                format!("{} ; {}", s1.show(e), s2.show(e))
            }
        }
    }
}

impl<A: Clone, O: Clone> pbn::Step for Step<A, O> {
    type Exp = Exp<A, O>;

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Step::Add(oid, _) => {
                let mut ret = e.clone();
                if ret.allowed.set.insert(*oid) {
                    Some(ret)
                } else {
                    None
                }
            }
            Step::Reject(oid) => {
                let mut ret = e.clone();
                if ret.rejected.set.insert(*oid) {
                    Some(ret)
                } else {
                    None
                }
            }
            Step::Refine(x, axs) => {
                let mut ret = e.clone();
                if ret.allowed.set.swap_remove(x) {
                    ret.allowed.set.extend(axs.set.iter().cloned());
                    Some(ret)
                } else {
                    None
                }
            }
            Step::Commit(oid) => {
                let mut ret = e.clone();
                if !ret.allowed.set.swap_remove(oid) {
                    return None;
                }
                if !ret.committed.set.insert(*oid) {
                    return None;
                }
                Some(ret)
            }
            Step::Seq(s1, s2) => s1.apply(e).and_then(|x| s2.apply(&x)),
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
        ao::algo::provable_with(&self.graph, &e.committed)
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
// Committal add provider

pub enum CommittalAddAlgorithm {
    Naive,
}

pub struct CommittalAddProvider<A, O> {
    phantom: PhantomData<(A, O)>,
    proper_axiom_sets: Vec<ao::NodeSet>,
    algorithm: CommittalAddAlgorithm,
}

impl<A: Clone, O: Clone> CommittalAddProvider<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Self {
            phantom: PhantomData,
            proper_axiom_sets: ao::algo::proper_axiom_sets(&graph),
            algorithm: CommittalAddAlgorithm::Naive,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for CommittalAddProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut next_labels = IndexSet::new();

        for axs in &self.proper_axiom_sets {
            if e.committed.set.is_subset(&axs.set) {
                next_labels
                    .extend(axs.set.difference(&e.committed.set).cloned())
            }
        }

        let steps: Vec<_> = next_labels
            .into_iter()
            .map(|o| {
                Step::Seq(
                    Box::new(Step::Add(o, PhantomData)),
                    Box::new(Step::Commit(o)),
                )
            })
            .collect();

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Naive refine provider
// Note: Only operates on allowed set, so requires some form of CommitProvider!

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

        for x in &e.allowed.set {
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

////////////////////////////////////////////////////////////////////////////////
// Basic commit provider

pub struct BasicCommitProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> BasicCommitProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for BasicCommitProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];

        for x in &e.allowed.set {
            steps.push(Step::Commit(*x))
        }

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Basic commit provider

pub struct ArbitrarySubsetCommitProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> ArbitrarySubsetCommitProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider
    for ArbitrarySubsetCommitProvider<A, O>
{
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        if !ao::algo::provable_with(&e.graph, &e.committed) {
            return Ok(vec![]);
        }

        let mut subset = e.committed.clone();

        'fixpoint: loop {
            for x in &subset.set {
                let mut candidate = subset.clone();
                candidate.set.swap_remove(x);
                if ao::algo::provable_with(&e.graph, &candidate) {
                    subset = candidate;
                    continue 'fixpoint;
                }
            }
            break;
        }

        let step =
            match Step::sequence(subset.set.into_iter().map(Step::Commit)) {
                Some(x) => x,
                None => return Ok(vec![]),
            };

        Ok(vec![step])
    }
}

use crate::ao;
use crate::pbn;
use crate::util::{self, EarlyCutoff, Timer};

use indexmap::IndexSet;
use std::hash::Hash;
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// Expressions

// U: unseen
// O: don't know if should be true or false
// F: should be false
// T: should be true
// T!: should be true; only consider solutions with it in dependencies
// T*: T + user will provide an impl
// T!*: T! + user will provide an impl
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum PartitionClass {
    // U
    Unseen,

    // O
    Unknown,

    // F
    ShouldBeFalse,

    // T
    ShouldBeTrue {
        will_provide: bool, // *
        force_use: bool,    // !
    },
}

impl PartitionClass {
    pub fn all() -> &'static [PartitionClass] {
        &[
            Self::Unseen,
            Self::Unknown,
            Self::ShouldBeFalse,
            Self::ShouldBeTrue {
                will_provide: false,
                force_use: false,
            },
            Self::ShouldBeTrue {
                will_provide: false,
                force_use: true,
            },
            Self::ShouldBeTrue {
                will_provide: true,
                force_use: false,
            },
            Self::ShouldBeTrue {
                will_provide: false,
                force_use: true,
            },
        ]
    }

    pub fn committed(&self) -> bool {
        match self {
            PartitionClass::ShouldBeTrue {
                will_provide,
                force_use,
            } => *will_provide && *force_use,
            PartitionClass::Unseen
            | PartitionClass::Unknown
            | PartitionClass::ShouldBeFalse => false,
        }
    }

    pub fn shorthand(&self) -> &str {
        match self {
            Self::Unseen => "U",
            Self::Unknown => "O",
            Self::ShouldBeFalse => "F",
            Self::ShouldBeTrue {
                will_provide: false,
                force_use: false,
            } => "T",
            Self::ShouldBeTrue {
                will_provide: false,
                force_use: true,
            } => "T!",
            Self::ShouldBeTrue {
                will_provide: true,
                force_use: false,
            } => "T*",
            Self::ShouldBeTrue {
                will_provide: true,
                force_use: true,
            } => "T!*",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Partitioned<T> {
    data: T,
    class: PartitionClass,
}

#[derive(Debug, Clone)]
pub struct Exp<A, O> {
    graph: ao::Graph<A, Partitioned<O>>,
}

impl<A: Clone, O: Clone> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        Exp {
            graph: graph.map(
                |o| {
                    o.map(|data| Partitioned {
                        data: data.clone(),
                        class: PartitionClass::Unseen,
                    })
                },
                |a| a.cloned(),
            ),
        }
    }

    pub fn graph(&self) -> &ao::Graph<A, Partitioned<O>> {
        &self.graph
    }

    pub fn committed(&self) -> ao::NodeSet {
        ao::NodeSet {
            set: self
                .graph
                .or_indexes()
                .filter(|oidx| {
                    self.graph.or_data_ref(*oidx).unwrap().class.committed()
                })
                .collect(),
        }
    }
}

impl<A, O> std::fmt::Display for Exp<A, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (class, oidxs) in util::preimage(
            self.graph
                .or_indexes()
                .map(|oid| (oid, self.graph.or_data_ref(oid).unwrap().class)),
        ) {
            let ns = ao::NodeSet {
                set: oidxs.into_iter().collect(),
            };
            write!(f, "{}: {} ", class.shorthand(), ns.show(&self.graph))?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Steps

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<A, O> {
    /// Set a node's partition class
    SetClass(ao::OIdx, PartitionClass, PhantomData<(A, O)>),

    /// Refine a node the user said they would provide
    Refine(ao::OIdx, ao::NodeSet),

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
            Step::SetClass(oid, class, _) => {
                format!("{} += {}", class.shorthand(), e.graph.or_at(*oid))
            }
            Step::Refine(oid, axs) => {
                format!("{} -> {}", e.graph.or_at(*oid), axs.show(&e.graph))
            }
            Step::Seq(s1, s2) => {
                format!("{} ; {}", s1.show(e), s2.show(e))
            }
        }
    }
}

impl<A: Clone, O: Clone> pbn::Step for Step<A, O> {
    type Exp = Exp<A, O>;

    // TODO Enforce partition class size invariants
    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Step::SetClass(oid, class, _) => {
                let mut ret = e.clone();
                ret.graph.or_data_mut(*oid).unwrap().class = *class;
                Some(ret)
            }
            Step::Refine(oid, ns) => {
                let mut ret = e.clone();
                let data = ret.graph.or_data_mut(*oid).unwrap();
                if data.class.committed() {
                    data.class = PartitionClass::ShouldBeTrue {
                        will_provide: false,
                        force_use: true,
                    };
                    for new_oid in &ns.set {
                        ret.graph.or_data_mut(*new_oid).unwrap().class =
                            PartitionClass::ShouldBeTrue {
                                will_provide: true,
                                force_use: true,
                            }
                    }
                    Some(ret)
                } else {
                    None
                }
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
        ao::algo::provable_with(&self.graph, &e.committed())
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

        let committed = e.committed();
        for axs in &self.proper_axiom_sets {
            if committed.set.is_subset(&axs.set) {
                next_labels.extend(axs.set.difference(&committed.set).cloned())
            }
        }

        let steps: Vec<_> = next_labels
            .into_iter()
            .map(|o| {
                Step::SetClass(
                    o,
                    PartitionClass::ShouldBeTrue {
                        will_provide: true,
                        force_use: true,
                    },
                    PhantomData,
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

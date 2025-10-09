use crate::ao;
use crate::pbn;
use crate::util::{self, EarlyCutoff, Timer};

use indexmap::IndexSet;
use rand::prelude::*;
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
#[allow(dead_code)]
pub enum PartitionClass {
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
    #[allow(dead_code)]
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

    pub fn unseen(&self) -> bool {
        match self {
            PartitionClass::Unseen => true,
            PartitionClass::ShouldBeTrue { .. }
            | PartitionClass::Unknown
            | PartitionClass::ShouldBeFalse => false,
        }
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

    pub fn provided(&self) -> bool {
        match self {
            PartitionClass::ShouldBeTrue { will_provide, .. } => *will_provide,
            PartitionClass::Unseen
            | PartitionClass::Unknown
            | PartitionClass::ShouldBeFalse => false,
        }
    }

    pub fn should_be_true(&self) -> bool {
        match self {
            PartitionClass::ShouldBeTrue { .. } => true,
            PartitionClass::Unseen
            | PartitionClass::Unknown
            | PartitionClass::ShouldBeFalse => false,
        }
    }

    pub fn should_be_false(&self) -> bool {
        match self {
            PartitionClass::ShouldBeFalse => true,
            PartitionClass::Unseen
            | PartitionClass::Unknown
            | PartitionClass::ShouldBeTrue { .. } => false,
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
    #[allow(dead_code)]
    data: T,
    class: PartitionClass,
}

#[derive(Debug, Clone)]
pub struct Exp<A, O> {
    graph: ao::Graph<A, Partitioned<Option<O>>>,
}

impl<A: Clone, O: Clone> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        let mut graph = graph.map(
            |o| {
                Some(Partitioned {
                    data: o.cloned(),
                    class: PartitionClass::Unseen,
                })
            },
            |a| a.cloned(),
        );

        graph.or_data_mut(graph.goal()).unwrap().class =
            PartitionClass::ShouldBeTrue {
                will_provide: false,
                force_use: true,
            };

        Self { graph }
    }

    pub fn graph(&self) -> &ao::Graph<A, Partitioned<Option<O>>> {
        &self.graph
    }

    pub fn unseen(&self) -> ao::NodeSet {
        ao::NodeSet {
            set: self
                .graph
                .or_indexes()
                .filter(|oidx| {
                    self.graph.or_data_ref(*oidx).unwrap().class.unseen()
                })
                .collect(),
        }
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

    pub fn provided(&self) -> ao::NodeSet {
        ao::NodeSet {
            set: self
                .graph
                .or_indexes()
                .filter(|oidx| {
                    self.graph.or_data_ref(*oidx).unwrap().class.provided()
                })
                .collect(),
        }
    }

    pub fn should_be_true(&self) -> ao::NodeSet {
        ao::NodeSet {
            set: self
                .graph
                .or_indexes()
                .filter(|oidx| {
                    self.graph
                        .or_data_ref(*oidx)
                        .unwrap()
                        .class
                        .should_be_true()
                })
                .collect(),
        }
    }

    pub fn should_be_false(&self) -> ao::NodeSet {
        ao::NodeSet {
            set: self
                .graph
                .or_indexes()
                .filter(|oidx| {
                    self.graph
                        .or_data_ref(*oidx)
                        .unwrap()
                        .class
                        .should_be_false()
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

fn consistent_with<A: Clone, O: Clone>(
    e: &Exp<A, O>,
    extra_provided: &ao::NodeSet,
    unprovided: &ao::NodeSet,
) -> bool {
    let mut g = e.graph().clone();

    g.make_axioms(
        e.provided()
            .set
            .union(&extra_provided.set)
            .cloned()
            .collect::<IndexSet<_>>()
            .difference(&unprovided.set)
            .cloned(),
    );
    g.force_all_false(e.should_be_false().set.into_iter());

    let provable = ao::algo::provable_or_nodes(&g);

    e.should_be_true().set.is_subset(&provable.set)
}

fn consistent<A: Clone, O: Clone>(e: &Exp<A, O>) -> bool {
    consistent_with(e, &ao::NodeSet::new(), &ao::NodeSet::new())
}

fn valid<A: Clone, O: Clone>(e: &Exp<A, O>) -> bool {
    // Check consistent
    if !consistent(e) {
        return false;
    }

    // Check proper (TODO: this check is not right because provided nodes are
    // classified as "should be true", so when they are unprovided, they (can)
    // become unprovable, which will make e inconsistent.)
    // for p in e.provided().set {
    //     if consistent_with(e, &ao::NodeSet::new(), &ao::NodeSet::singleton(p)) {
    //         return false;
    //     }
    // }

    return true;
}

pub struct GoalProvable<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> GoalProvable<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::ValidityChecker for GoalProvable<A, O> {
    type Exp = Exp<A, O>;

    fn check(&self, e: &Self::Exp) -> bool {
        valid(e)
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
    #[allow(dead_code)]
    algorithm: CommittalAddAlgorithm,
}

impl<A: Clone, O: Clone> CommittalAddProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            algorithm: CommittalAddAlgorithm::Naive,
        }
    }
}

// TODO make compatible with "remove" steps
impl<A: Clone, O: Clone> pbn::StepProvider for CommittalAddProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut next_labels = IndexSet::new();

        let committed = e.committed();
        for axs in ao::algo::proper_axiom_sets(e.graph(), e.graph().goal()) {
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
// Complete refine provider

pub struct CompleteRefineProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> CompleteRefineProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for CompleteRefineProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];

        for x in e.committed().set {
            for axs in ao::algo::proper_axiom_sets(e.graph(), x) {
                if axs.set == IndexSet::from([x]) {
                    continue;
                }
                steps.push(Step::Refine(x, axs))
            }
        }

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Arbitrary subset commit provider

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
        if valid(e) {
            return Ok(vec![]);
        }

        let mut subset = e.provided();

        if !consistent_with(e, &subset, &ao::NodeSet::new()) {
            return Ok(vec![]);
        }

        'fixpoint: loop {
            for x in &subset.set {
                let c = e.graph.or_data_ref(*x).unwrap().class;
                if c.provided() && !c.committed() {
                    let mut candidate = subset.clone();
                    candidate.set.swap_remove(x);
                    if consistent_with(e, &candidate, &ao::NodeSet::new()) {
                        subset = candidate;
                        continue 'fixpoint;
                    }
                }
            }
            break;
        }

        let step = match Step::sequence(subset.set.into_iter().map(|oid| {
            Step::SetClass(
                oid,
                PartitionClass::ShouldBeTrue {
                    will_provide: true,
                    force_use: true,
                },
                PhantomData,
            )
        })) {
            Some(x) => x,
            None => return Ok(vec![]),
        };

        Ok(vec![step])
    }
}

////////////////////////////////////////////////////////////////////////////////
// Random provider

pub struct RandomProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> RandomProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for RandomProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut rest = e.unseen().set;
        if rest.is_empty() {
            return Ok(vec![]);
        }
        let oid = rest[rand::rng().random_range(0..rest.len())];
        let _ = rest.swap_remove(&oid);

        let mut ret = vec![Step::SetClass(
            oid,
            PartitionClass::ShouldBeTrue {
                will_provide: true,
                force_use: false,
            },
            PhantomData,
        )];

        if !consistent_with(
            e,
            &ao::NodeSet {
                set: e.committed().set.union(&rest).cloned().collect(),
            },
            &ao::NodeSet::new(),
        ) {
            return Ok(ret);
        }

        ret.push(Step::SetClass(
            oid,
            PartitionClass::ShouldBeFalse,
            PhantomData,
        ));

        ret.push(Step::SetClass(
            oid,
            PartitionClass::ShouldBeTrue {
                will_provide: false,
                force_use: false,
            },
            PhantomData,
        ));

        Ok(ret)
    }
}

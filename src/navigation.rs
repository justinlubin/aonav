use crate::ao;
use crate::pbn;
use crate::util;

use indexmap::IndexSet;
use rand::prelude::*;
use std::hash::Hash;
use std::marker::PhantomData;

pub mod providers;

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
                format!("set {} {}", e.graph.or_at(*oid), class.shorthand())
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

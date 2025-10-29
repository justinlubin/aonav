use crate::ao;
use crate::pbn;
use crate::util;

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
#[allow(dead_code)]
pub enum Class {
    // ⊥ ("Bot")
    Unseen,

    // ?
    Unknown,

    // F
    False,

    // T
    True { force_use: bool },

    // A
    Assume { force_use: bool },
}

impl Class {
    #[allow(dead_code)]
    pub fn all() -> &'static [Class] {
        &[
            Self::Unseen,
            Self::Unknown,
            Self::False,
            Self::True { force_use: false },
            Self::True { force_use: true },
            Self::Assume { force_use: false },
            Self::Assume { force_use: true },
        ]
    }

    // pub fn committed(&self) -> bool {
    //     match self {
    //         PartitionClass::ShouldBeTrue {
    //             will_provide,
    //             force_use,
    //         } => *will_provide && *force_use,
    //         PartitionClass::Unseen
    //         | PartitionClass::Unknown
    //         | PartitionClass::ShouldBeFalse => false,
    //     }
    // }

    // pub fn provided(&self) -> bool {
    //     match self {
    //         PartitionClass::ShouldBeTrue { will_provide, .. } => *will_provide,
    //         PartitionClass::Unseen
    //         | PartitionClass::Unknown
    //         | PartitionClass::ShouldBeFalse => false,
    //     }
    // }

    // pub fn should_be_true(&self) -> bool {
    //     match self {
    //         PartitionClass::ShouldBeTrue { .. } => true,
    //         PartitionClass::Unseen
    //         | PartitionClass::Unknown
    //         | PartitionClass::ShouldBeFalse => false,
    //     }
    // }

    // pub fn should_be_false(&self) -> bool {
    //     match self {
    //         PartitionClass::ShouldBeFalse => true,
    //         PartitionClass::Unseen
    //         | PartitionClass::Unknown
    //         | PartitionClass::ShouldBeTrue { .. } => false,
    //     }
    // }

    pub fn shorthand(&self) -> &str {
        match self {
            Self::Unseen => "⊥",
            Self::Unknown => "?",
            Self::False => "F",
            Self::True { force_use: false } => "T",
            Self::True { force_use: true } => "T!",
            Self::Assume { force_use: false } => "A",
            Self::Assume { force_use: true } => "A!",
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            Self::True { .. } => true,
            _ => false,
        }
    }

    pub fn is_assume(&self) -> bool {
        match self {
            Self::Assume { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Partitioned<T> {
    #[allow(dead_code)]
    pub data: T,
    pub class: Class,
}

impl<T> Partitioned<T> {
    pub fn as_ref(&self) -> Partitioned<&T> {
        Partitioned {
            data: &self.data,
            class: self.class,
        }
    }

    pub fn as_mut(&mut self) -> Partitioned<&mut T> {
        Partitioned {
            data: &mut self.data,
            class: self.class,
        }
    }

    pub fn map<U, F>(self, f: F) -> Partitioned<U>
    where
        F: FnOnce(T) -> U,
    {
        Partitioned {
            data: f(self.data),
            class: self.class,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Exp<A, O> {
    graph: ao::Graph<A, Partitioned<Option<O>>>,
}

impl<A, O> Exp<A, O> {
    pub fn new(graph: ao::Graph<A, O>) -> Self {
        let mut graph = graph.map_owned(
            |o| {
                Some(Partitioned {
                    data: o,
                    class: PartitionClass::Unseen,
                })
            },
            |a| a,
        );

        let mut ret = Self { graph };

        ret.or_data_mut(ret.graph().goal()).class =
            PartitionClass::True { force_use: true };

        ret
    }

    pub fn graph(&self) -> &ao::Graph<A, Partitioned<Option<O>>> {
        &self.graph
    }

    pub fn or_data_ref(&self, o: OIdx) -> Partitioned<Option<&O>> {
        self.graph()
            .or_data_ref(o)
            .unwrap()
            .as_ref()
            .map(|x| x.as_ref())
    }

    pub fn or_data_mut(&mut self, o: OIdx) -> Partitioned<Option<&mut O>> {
        self.graph()
            .or_data_mut(o)
            .unwrap()
            .as_mut()
            .map(|x| x.as_mut())
    }

    pub fn class_mut(&mut self, o: OIdx) -> &mut Class {
        &mut self.graph().or_data_mut(o).unwrap().class
    }

    pub fn or_indexes_by_class<F>(&self, f: F) -> ao::NodeSet
    where
        F: Fn(Class) -> bool,
    {
        ao::NodeSet {
            set: self
                .graph()
                .or_indexes()
                .filter(|oidx| f(self.or_data_ref(*oidx).class))
                .collect(),
        }
    }
}

impl<A, O> std::fmt::Display for Exp<A, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for class in Class::all() {
            let ns = self.or_indexes_by_class(|c| c == *class);
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

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Step::SetClass(oid, c, _) => {
                let mut ret = e.clone();
                let class_ref = ret.class_mut(*oid);
                if *class_ref != Class::Unseen {
                    return None;
                }
                *class_ref = c;
                Some(ret)
            }
            Step::Seq(s1, s2) => s1.apply(e).and_then(|e2| s2.apply(&e2)),
        }
    }
}

// TODO implement sorting for steps

////////////////////////////////////////////////////////////////////////////////
// Validity checker

// pub fn consistent_with<A: Clone, O: Clone>(
//     e: &Exp<A, O>,
//     extra_provided: &ao::NodeSet,
//     unprovided: &ao::NodeSet,
// ) -> bool {
//     let mut g = e.graph().clone();
//
//     g.make_axioms(
//         e.provided()
//             .set
//             .union(&extra_provided.set)
//             .cloned()
//             .collect::<IndexSet<_>>()
//             .difference(&unprovided.set)
//             .cloned(),
//     );
//     g.force_all_false(e.should_be_false().set.into_iter());
//
//     let provable = ao::algo::provable_or_nodes(&g);
//
//     e.should_be_true().set.is_subset(&provable.set)
// }
//
// fn consistent<A: Clone, O: Clone>(e: &Exp<A, O>) -> bool {
//     consistent_with(e, &ao::NodeSet::new(), &ao::NodeSet::new())
// }

fn prune_forced<A, O>(g: ao::Graph<A, O>) -> ao::Graph<A, O> {
    todo!()
}

pub fn valid<A: Clone, O: Clone>(e: &Exp<A, O>) -> bool {
    // Treat bot as ?
    let mut g = e.graph().clone();

    // Add axioms for A / A!
    g.make_axioms(e.or_indexes_by_class(|c| c.is_assume()).set.into_iter());

    // Compute all provable nodes - make sure contains T / T! and disjoint with F
    let provable = ao::algo::provable_or_nodes(&g);

    // Make sure contains T / T! ...

    let contains_true = provable
        .set
        .is_superset(&e.or_indexes_by_class(|c| c.is_true()).set);

    if !contains_true {
        return false;
    }

    // ... and disjoint from F
    let disjoint_from_false = provable
        .set
        .is_disjoint(&e.or_indexes_by_class(|c| c == Class::False).set);

    if !disjoint_from_false {
        return false;
    }

    // Prune according to ! nodes, check if goal still provable

    let pruned = prune_forced(e.graph.clone());

    ao::algo::provable(&pruned, pruned.goal())
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

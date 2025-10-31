use crate::ao;
use crate::pbn;

use indexmap::IndexMap;
use std::hash::Hash;

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
pub struct Exp {
    graph: ao::Graph,
    partition: IndexMap<ao::OIdx, Class>,
}

impl Exp {
    pub fn new(graph: ao::Graph) -> Self {
        let mut partition: IndexMap<_, _> = graph
            .or_indexes()
            .map(|oidx| (oidx, Class::Unseen))
            .collect();
        *partition.get_mut(&graph.goal()).unwrap() =
            Class::True { force_use: true };
        Self { graph, partition }
    }

    pub fn graph(&self) -> &ao::Graph {
        &self.graph
    }

    pub fn partition(&self) -> &IndexMap<ao::OIdx, Class> {
        &self.partition
    }

    pub fn class(&self, oidx: ao::OIdx) -> Class {
        *self.partition().get(&oidx).unwrap()
    }

    pub fn filter_class<F>(&self, f: F) -> ao::OrSet
    where
        F: Fn(Class) -> bool,
    {
        ao::OrSet {
            set: self
                .partition()
                .iter()
                .filter_map(
                    |(oidx, class)| if f(*class) { Some(*oidx) } else { None },
                )
                .collect(),
        }
    }
}

impl std::fmt::Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for class in Class::all() {
            let os = self.filter_class(|c| c == *class);
            write!(f, "{}: {}    ", class.shorthand(), os.show(&self.graph))?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Steps

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Set a node's partition class
    SetClass(ao::OIdx, Class, Option<String>),

    /// Sequence two steps
    Seq(Box<Step>, Box<Step>, Option<String>),
}

impl Step {
    pub fn set_label(&mut self, label: Option<String>) {
        match self {
            Self::SetClass(_, _, old_label) => *old_label = label,
            Self::Seq(_, _, old_label) => *old_label = label,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            Self::SetClass(_, _, label) => label.as_ref().map(|x| x.as_str()),
            Self::Seq(_, _, label) => label.as_ref().map(|x| x.as_str()),
        }
    }

    pub fn sequence(mut steps: impl Iterator<Item = Step>) -> Option<Step> {
        let mut step = steps.next()?;

        for s in steps {
            step = Step::Seq(Box::new(step), Box::new(s), None);
        }

        Some(step)
    }

    pub fn show(&self, e: &Exp) -> String {
        let default = match self {
            Step::SetClass(oid, class, _) => {
                format!(
                    "set \"{}\" to {}",
                    e.graph.or_at(*oid),
                    class.shorthand()
                )
            }
            Step::Seq(s1, s2, _) => {
                format!("{} ; {}", s1.show(e), s2.show(e))
            }
        };

        match self.label() {
            None => default,
            Some(lab) => {
                format!(
                    "{} {}",
                    lab,
                    ansi_term::Color::Fixed(8).paint(format!("({})", default))
                )
            }
        }
    }
}

impl pbn::Step for Step {
    type Exp = Exp;

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Step::SetClass(oid, c, _) => {
                let mut ret = e.clone();
                if *ret.partition.get(oid).unwrap() != Class::Unseen {
                    return None;
                }
                *ret.partition.get_mut(oid).unwrap() = *c;
                Some(ret)
            }
            Step::Seq(s1, s2, _) => s1.apply(e).and_then(|e2| s2.apply(&e2)),
        }
    }
}

// TODO implement sorting for steps

////////////////////////////////////////////////////////////////////////////////
// Validity checker

// TODO
// fn prune_forced(g: ao::Graph) -> ao::Graph {
// }

pub fn valid(e: &Exp) -> bool {
    let mut g = e.graph().clone();

    // Add axioms for A / A!

    g.make_axioms(e.filter_class(|c| c.is_assume()).set.into_iter());

    // Compute all provable nodes

    let provable = ao::algo::provable_or_nodes(&g);

    // Make sure contains T / T! ...

    let contains_true = provable
        .set
        .is_superset(&e.filter_class(|c| c.is_true()).set);

    if !contains_true {
        return false;
    }

    // ... and disjoint from F

    let disjoint_from_false = provable
        .set
        .is_disjoint(&e.filter_class(|c| c == Class::False).set);

    if !disjoint_from_false {
        return false;
    }

    true

    // TODO: Prune according to ! nodes, check if goal still provable
    // ao::algo::provable(&g, g.goal())
}

pub struct Valid;

impl Valid {
    pub fn new() -> Self {
        Self {}
    }
}

impl pbn::ValidityChecker for Valid {
    type Exp = Exp;

    fn check(&self, e: &Self::Exp) -> bool {
        valid(e)
    }
}

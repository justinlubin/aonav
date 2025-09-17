use crate::ao;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;

////////////////////////////////////////////////////////////////////////////////
// Basics

// Expressions

#[derive(Debug, Clone)]
pub struct AxiomSet(IndexSet<String>);

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
    Add(String),
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

pub struct TargetReachableChecker {
    target: String,
}

impl TargetReachableChecker {
    pub fn new(target: String) -> Self {
        Self { target }
    }
}

impl pbn::ValidityChecker for TargetReachableChecker {
    type Exp = AxiomSet;

    fn check(&self, _e: &Self::Exp) -> bool {
        // TODO implement this!
        true
    }
}

////////////////////////////////////////////////////////////////////////////////
// Providers

pub struct IncorrectProvider {
    pub graph: ao::AndOrGraph<String, String>,
}

impl pbn::StepProvider for IncorrectProvider {
    type Step = AOStep;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &AxiomSet,
    ) -> Result<Vec<AOStep>, EarlyCutoff> {
        let mut steps = vec![];

        for node in &self.graph.or_nodes() {
            if e.0.contains(node) {
                continue;
            }
            steps.push(AOStep::Add(node.clone()))
        }

        Ok(steps)
    }
}

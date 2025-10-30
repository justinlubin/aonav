//! # Programming By Navigation
//!
//! This module defines all the necessary high-level components of Programming
//! By Navigation. In particular, it defines the interface that is necessary for
//! the Programming By Navigation interaction and guarantees.

use crate::util::{EarlyCutoff, Timer};

/// The type of steps.
///
/// Steps transform one expression into another and must satisfy the
/// *navigation relation* properties.
pub trait Step {
    type Exp: Clone;
    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

pub trait ValidityChecker {
    type Exp;
    fn check(&self, e: &Self::Exp) -> bool;
}

/// The type of step providers.
///
/// To be a valid solution to the Programming By Navigation Synthesis Problem,
/// step providers must satisfy the *validity*, *strong completeness*, and
/// *strong soundness* conditions.
pub trait StepProvider {
    type Step: Step;
    fn provide(
        &mut self,
        timer: &Timer,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff>;
}

/// Compound provider (composition of other providers)
pub struct CompoundProvider<S: Step> {
    providers: Vec<Box<dyn StepProvider<Step = S>>>,
}

impl<S: Step> CompoundProvider<S> {
    pub fn new(providers: Vec<Box<dyn StepProvider<Step = S>>>) -> Self {
        Self { providers }
    }
}

impl<S: Step> StepProvider for CompoundProvider<S> {
    type Step = S;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];
        for p in &mut self.providers {
            steps.extend(p.provide(timer, e)?);
        }
        Ok(steps)
    }
}

/// A Programming By Navigation controller. Controllers abstract away the
/// actual step provider (and validity checker) and can be used to engage in
/// the Programming By Navigation interactive process in a way that is
/// abstracted from the underlying synthesis algorithm.
pub struct Controller<S: Step> {
    timer: Timer,
    provider: Box<dyn StepProvider<Step = S> + 'static>,
    checker: Box<dyn ValidityChecker<Exp = S::Exp> + 'static>,
    state: S::Exp,
    history: Option<Vec<S::Exp>>,
}

impl<S: Step> Controller<S> {
    /// Create a new controller
    pub fn new(
        timer: Timer,
        provider: impl StepProvider<Step = S> + 'static,
        checker: impl ValidityChecker<Exp = S::Exp> + 'static,
        start: S::Exp,
        save_history: bool,
    ) -> Self {
        Self {
            timer,
            provider: Box::new(provider),
            checker: Box::new(checker),
            state: start,
            history: if save_history { Some(vec![]) } else { None },
        }
    }

    /// Ask the synthesizer to provide a list of possible next steps (all and
    /// only the valid ones)
    pub fn provide(&mut self) -> Result<Vec<S>, EarlyCutoff> {
        self.provider.provide(&self.timer, &self.state)
    }

    /// Decide which step to take - must be selected from among the ones that
    /// are provided by the [`provide`] function
    pub fn decide(&mut self, step: S) {
        match &mut self.history {
            None => (),
            Some(his) => his.push(self.state.clone()),
        };
        self.state = step.apply(&self.state).unwrap();
    }

    // TODO consider returning reference?
    /// Returns the current working expression
    pub fn working_expression(&self) -> S::Exp {
        self.state.clone()
    }

    /// Returns whether or not the current working expression is valid
    pub fn valid(&self) -> bool {
        self.checker.check(&self.state)
    }

    /// Returns whether or not "undo" is applicable
    pub fn can_undo(&self) -> bool {
        match &self.history {
            None => false,
            Some(xs) => !xs.is_empty(),
        }
    }

    /// Perform an "undo" (panic if not possible)
    pub fn undo(&mut self) {
        self.state = self.history.as_mut().unwrap().pop().unwrap();
    }
}

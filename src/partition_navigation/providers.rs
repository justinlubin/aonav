//! # Step Providers for Programming by Navigation
//!
//! Defined in Section 5 and Appendix B of the paper

use std::collections::HashMap;

use crate::partition_navigation as pn;
use crate::util::{self, EarlyCutoff, Timer};
use pn::oracle::OptInc;

use aograph::OIdx;
use indexmap::IndexSet;
use pbn::{Step, Timer as _, ValidityChecker};
use rand::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// Commit

/// Provides steps to commit non-committed nodes
pub struct Commit {
    incremental: OptInc,
}

impl Commit {
    /// Create an instance of the "Commit" Step Provider
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::StepProvider<util::Timer> for Commit {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];

        for (oidx, class) in e.partition() {
            for assume in &[false, true] {
                let new_class = match class.commit_true(*assume) {
                    Some(nc) => nc,
                    None => continue,
                };

                let step = pn::Step::SetClass(*oidx, new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if self.incremental.nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }

                timer.tick()?;
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Remaining

/// Shows all valid labels for all remaining (not-labeled) nodes
pub struct Remaining {
    incremental: OptInc,
    committed_only: bool,
}

impl Remaining {
    /// Create an instance of the "Remaining" Step Provider
    pub fn new(incremental: OptInc, committed_only: bool) -> Self {
        Self {
            incremental,
            committed_only,
        }
    }
}

impl pbn::StepProvider<util::Timer> for Remaining {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let choices = if self.committed_only {
            pn::Class::committed()
        } else {
            pn::Class::all()
        };

        let mut ret = vec![];
        for oidx in e.partition().keys() {
            for new_class in choices {
                let step = pn::Step::SetClass(*oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if self.incremental.nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
                timer.tick()?;
            }
        }
        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Random

/// Selects a node at random and provides all valid labels for that node
pub struct Random {
    incremental: OptInc,
}

impl Random {
    /// Create an instance of the "Random" Step Provider
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::StepProvider<util::Timer> for Random {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let unseen = e.filter_class(|c| !c.is_committed()).set;
        if unseen.is_empty() {
            return Ok(vec![]);
        }

        let oidx = unseen[rand::rng().random_range(0..unseen.len())];

        let mut ret = vec![];

        for new_class in pn::Class::committed() {
            let step = pn::Step::SetClass(oidx, *new_class, None);
            match step.apply(e) {
                None => continue,
                Some(result) => {
                    if self.incremental.nonempty_completion(&result) {
                        ret.push(step);
                    }
                }
            }
            timer.tick()?;
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Top-down inversion

/// Provides steps to iteratively traverse the graph using a top-down strategy
pub struct TopDownInversion {
    incremental: OptInc,
}

impl TopDownInversion {
    /// Create an instance of the "TopDownInversion" Step Provider
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::StepProvider<util::Timer> for TopDownInversion {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];

        let frontier = e
            .filter_class(|c| {
                c == pn::Class::True {
                    force_use: true,
                    assume: None,
                } || c
                    == pn::Class::True {
                        force_use: true,
                        assume: Some(false),
                    }
            })
            .set;

        for oidx in frontier {
            for aidx in e.graph().providers(oidx) {
                let unseen = e
                    .graph()
                    .premises(aidx)
                    .filter(|&prem_oidx| {
                        e.class(prem_oidx) == pn::Class::Unseen
                    })
                    .collect::<IndexSet<_>>();

                if unseen.is_empty() {
                    continue;
                }

                let it = unseen.into_iter().map(|prem_oidx| {
                    pn::Step::SetClass(
                        prem_oidx,
                        pn::Class::True {
                            force_use: true,
                            assume: None,
                        },
                        None,
                    )
                });

                let it: Box<dyn Iterator<Item = _>> = if e.class(oidx)
                    == (pn::Class::True {
                        force_use: true,
                        assume: None,
                    }) {
                    Box::new(it.chain(std::iter::once(pn::Step::SetClass(
                        oidx,
                        pn::Class::True {
                            force_use: true,
                            assume: Some(false),
                        },
                        None,
                    ))))
                } else {
                    Box::new(it)
                };

                let mut step = pn::Step::sequence(it).unwrap();

                step.set_label(Some(format!(
                    "explore rule \"{}\"",
                    e.graph().and_at(aidx)
                )));

                // Intuitively this should always apply and does NOT need oracle
                // call, but oracle call is necessary if want to combine with
                // other strategies
                if let Some(result) = step.apply(e) {
                    if self.incremental.nonempty_completion(&result) {
                        ret.push(step);
                    }
                }

                timer.tick()?;
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Bottom-up inversion

/// Provides steps to iteratively travese the graph using a bottom-up strategy
pub struct BottomUpInversion {
    incremental: OptInc,
}

impl BottomUpInversion {
    /// Create an instance of the "BottomUpInversion" Step Provider
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::StepProvider<util::Timer> for BottomUpInversion {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];

        let frontier = e.filter_class(|c| c == pn::Class::False).set;

        for oidx in frontier {
            for aidx in e.graph().consumers(oidx) {
                let conclusion_oidx = e.graph().conclusion(aidx);
                for new_class in pn::Class::all() {
                    let step =
                        pn::Step::SetClass(conclusion_oidx, *new_class, None);
                    if let Some(result) = step.apply(e) {
                        if self.incremental.nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                    timer.tick()?;
                }
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Leaf

/// Provides valid possible labels for leaf nodes
pub struct Leaf {
    incremental: OptInc,
}

impl Leaf {
    /// Create an instance of the "Leaf" Step Provider
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::StepProvider<util::Timer> for Leaf {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        for oidx in e.graph().or_leaves() {
            for new_class in pn::Class::committed() {
                let step = pn::Step::SetClass(oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if self.incremental.nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
                timer.tick()?;
            }
        }
        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Maximum information gain

/// Selects a node whose labelling has the highest expected information gain and
/// provides all valid labels for that node
pub struct MaxInfoGain {
    incremental: OptInc,
    relevancy_prune: bool,
}

impl MaxInfoGain {
    /// Create an instance of the "MaxInfoGain" Step Provider
    pub fn new(incremental: OptInc, relevancy_prune: bool) -> Self {
        Self {
            incremental,
            relevancy_prune,
        }
    }
}

impl pbn::StepProvider<util::Timer> for MaxInfoGain {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        let mut min_expected_entropy = f64::INFINITY;

        let projected = aograph::OrSet {
            set: if self.relevancy_prune {
                e.partition()
                    .keys()
                    .cloned()
                    .filter(|oidx| {
                        let force_step = pn::Step::SetClass(
                            *oidx,
                            pn::Class::True {
                                force_use: true,
                                assume: Some(true),
                            },
                            None,
                        );
                        match force_step.apply(e) {
                            None => false,
                            Some(force_result) => self
                                .incremental
                                .nonempty_completion(&force_result),
                        }
                    })
                    .collect()
            } else {
                e.partition().keys().cloned().collect()
            },
        };

        for oidx in &projected.set {
            let mut steps = vec![];
            let mut entropy_sum = 0.0;
            for new_class in pn::Class::committed() {
                let step = pn::Step::SetClass(*oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(child) => {
                        match pn::oracle::log10_projected_model_count(
                            &child, &projected,
                        )? {
                            Some(h) => {
                                entropy_sum += h;
                                steps.push(step);
                            }
                            None => {}
                        }
                    }
                }
                timer.tick()?;
            }
            if steps.is_empty() {
                continue;
            }
            let expected_entropy = entropy_sum / (steps.len() as f64);
            if expected_entropy < min_expected_entropy {
                min_expected_entropy = expected_entropy;
                ret = steps;
            }
        }
        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// MinLeafHeuristic

/// One-shot assigns an arbitrary set of leaf nodes to "assume" labels that
/// makes the goal provable
pub struct MinLeafHeuristic;

impl MinLeafHeuristic {
    // Create an instance of the "MinLeafHeuristic" Step Provider
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for MinLeafHeuristic {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let minimal_leaves = match pn::oracle::minimal_leaves(e) {
            Some(x) => x,
            None => return Ok(vec![]),
        };
        let possible_step =
            pn::Step::sequence(minimal_leaves.set.into_iter().map(|oidx| {
                pn::Step::SetClass(
                    oidx,
                    pn::Class::True {
                        force_use: false,
                        assume: Some(true),
                    },
                    None,
                )
            }));
        match possible_step {
            Some(step) => Ok(vec![step]),
            None => Ok(vec![]),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Forced assumptions

/// Shows only the OR nodes that must be assumed
pub struct ForcedAssumptions {
    provider: Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>>,
}

impl ForcedAssumptions {
    /// Create an instance of the "ForcedAssumptions" Step Provider
    pub fn new(
        provider: Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>>,
    ) -> Self {
        Self { provider }
    }
}

impl pbn::StepProvider<util::Timer> for ForcedAssumptions {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let steps = self.provider.provide(timer, e)?;
        let mut show = HashMap::new();
        for step in &steps {
            match step {
                pn::Step::SetClass(
                    oidx,
                    pn::Class::True {
                        assume: None | Some(true),
                        ..
                    },
                    _,
                ) => match show.get(oidx) {
                    Some(_) => (),
                    None => {
                        let _ = show.insert(*oidx, true);
                    }
                },
                pn::Step::SetClass(oidx, _, _) => {
                    let _ = show.insert(*oidx, false);
                }
                pn::Step::Seq(..) => (),
            };
        }
        Ok(steps
            .into_iter()
            .filter(|s| match s {
                pn::Step::SetClass(oidx, class, ..) => {
                    show.get(oidx) == Some(&true)
                        && match class {
                            pn::Class::True {
                                assume: Some(_), ..
                            } => true,
                            _ => false,
                        }
                }
                pn::Step::Seq(..) => false,
            })
            .collect())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Alphabetical

/// Variations of the "Alphabetical" Step Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphabeticalMode {
    Unsound,
    Complete,
    Relevant,
}

/// Provide all valid labels for the unseen OR-node that comes first
/// alphabetically; the Unsound mode also shows unsound labelings (violating
/// Strong Soundness, and the Relevant mode performs relevancy pruning,
/// satisfying only Strong Completeness Modulo Observability instead of full
/// Strong Completeness)
pub struct Alphabetical {
    incremental: OptInc,
    mode: AlphabeticalMode,
    fancy_prune: bool,
}

impl Alphabetical {
    /// Create an instance of the "Alphabetical" Step Provider
    pub fn new(incremental: OptInc, mode: AlphabeticalMode) -> Self {
        Self {
            incremental,
            mode,
            fancy_prune: false,
        }
    }
}

impl pbn::StepProvider<util::Timer> for Alphabetical {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut unseen: Vec<_> = e
            .filter_class(|c| !c.is_committed())
            .set
            .into_iter()
            .collect();
        unseen.sort_by(|o1, o2| {
            e.graph().or_at(*o1).id().cmp(e.graph().or_at(*o2).id())
        });

        for oidx in unseen {
            let mut ret = vec![];

            let mut show_unknown = true;

            if self.mode == AlphabeticalMode::Relevant {
                let force_step = pn::Step::SetClass(
                    oidx,
                    pn::Class::True {
                        force_use: true,
                        assume: Some(true),
                    },
                    None,
                );
                match force_step.apply(e) {
                    None => continue,
                    Some(force_result) => {
                        if !self.incremental.nonempty_completion(&force_result)
                        {
                            continue;
                        }
                    }
                }

                if self.fancy_prune {
                    let true_step = pn::Step::SetClass(
                        oidx,
                        pn::Class::True {
                            force_use: false,
                            assume: Some(false),
                        },
                        None,
                    );
                    let true_step_ok = match true_step.apply(e) {
                        None => false,
                        Some(result) => {
                            self.incremental.nonempty_completion(&result)
                        }
                    };

                    let false_step =
                        pn::Step::SetClass(oidx, pn::Class::False, None);
                    let false_step_ok = match false_step.apply(e) {
                        None => false,
                        Some(result) => {
                            self.incremental.nonempty_completion(&result)
                        }
                    };

                    show_unknown = true_step_ok && false_step_ok;
                }
            }

            for new_class in pn::Class::committed() {
                if !show_unknown && *new_class == pn::Class::Unknown {
                    continue;
                }
                let step = pn::Step::SetClass(oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if self.mode == AlphabeticalMode::Unsound
                            || self.incremental.nonempty_completion(&result)
                        {
                            ret.push(step);
                        }
                    }
                }
                timer.tick()?;
            }

            return Ok(ret);
        }
        return Ok(vec![]);
    }
}

////////////////////////////////////////////////////////////////////////////////
// Sufficiency Seeker

fn collate(steps: Vec<pn::Step>) -> HashMap<OIdx, Vec<pn::Step>> {
    let mut ret = HashMap::new();

    for step in steps {
        match step {
            pn::Step::SetClass(oidx, ..) => {
                ret.entry(oidx).or_insert_with(Vec::new).push(step)
            }
            pn::Step::Seq(..) => {
                panic!("collate does not support Seq steps")
            }
        }
    }

    ret
}

/// Identifies cut points in the graph and preferentially shows those OR nodes
/// to label
pub struct SufficiencySeeker {
    provider: Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>>,
    relevancy_prune: bool,
}

impl SufficiencySeeker {
    /// Create an instance of the "SufficiencySeeker" Step Provider
    pub fn new(
        provider: Box<dyn pbn::StepProvider<util::Timer, Step = pn::Step>>,
        relevancy_prune: bool,
    ) -> Self {
        Self {
            provider,
            relevancy_prune,
        }
    }
}

impl pbn::StepProvider<util::Timer> for SufficiencySeeker {
    type Step = pn::Step;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut collated_steps = collate(self.provider.provide(timer, e)?);
        let mut scores: Vec<(bool, f32, OIdx)> = vec![];

        for (oidx, steps) in &collated_steps {
            let mut relevant = false;
            let mut one_away = false;
            let mut assume_count = 0;

            for step in steps {
                if let pn::Step::SetClass(_, class, _) = step {
                    match class {
                        pn::Class::True {
                            assume: Some(true),
                            force_use,
                        } => {
                            assume_count += 1;
                            relevant |= force_use;
                        }
                        _ => (),
                    }
                    if class.is_assume() {
                        assume_count += 1;
                    }
                }
                if !one_away {
                    match step.apply(e) {
                        Some(result) => {
                            if pn::oracle::Sufficient::new().check(&result) {
                                one_away = true;
                            }
                        }
                        None => (),
                    }
                }
            }

            if self.relevancy_prune && !relevant {
                continue;
            }

            scores.push((
                one_away,
                assume_count as f32 / steps.len() as f32,
                *oidx,
            ));
        }

        match scores.iter().max_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.total_cmp(&b.1))
                .then(e.graph().or_at(a.2).id().cmp(e.graph().or_at(b.2).id()))
        }) {
            Some((_, _, oidx)) => Ok(collated_steps.remove(oidx).unwrap()),
            None => Ok(vec![]),
        }
    }
}

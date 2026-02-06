use std::collections::HashMap;

use crate::partition_navigation as pn;
use crate::util::{self, EarlyCutoff, Timer};

use indexmap::IndexSet;
use pbn::Step;
use rand::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// Commit

pub struct Commit;

impl Commit {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for Commit {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
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
                        if pn::oracle::nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Remaining

pub struct Remaining;

impl Remaining {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for Remaining {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        for oidx in e.partition().keys() {
            for new_class in pn::Class::all() {
                let step = pn::Step::SetClass(*oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if pn::oracle::nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
            }
        }
        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Random

pub struct Random;

impl Random {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for Random {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
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
                    if pn::oracle::nonempty_completion(&result) {
                        ret.push(step);
                    }
                }
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Top-down inversion

pub struct TopDownInversion;

impl TopDownInversion {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for TopDownInversion {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
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

                // TODO this should always apply and does NOT need oracle call
                // Although oracle call is necessary if want to combine with
                // other strategies
                if let Some(result) = step.apply(e) {
                    if pn::oracle::nonempty_completion(&result) {
                        ret.push(step);
                    }
                }
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Bottom-up inversion

pub struct BottomUpInversion;

impl BottomUpInversion {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for BottomUpInversion {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
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
                        if pn::oracle::nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
            }
        }

        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Leaf

pub struct Leaf;

impl Leaf {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for Leaf {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        for oidx in e.graph().or_leaves() {
            for new_class in pn::Class::committed() {
                let step = pn::Step::SetClass(oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(result) => {
                        if pn::oracle::nonempty_completion(&result) {
                            ret.push(step);
                        }
                    }
                }
            }
        }
        Ok(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Maximum information gain

pub struct MaxInfoGain {
    relevancy_prune: bool,
}

impl MaxInfoGain {
    pub fn new(relevancy_prune: bool) -> Self {
        Self { relevancy_prune }
    }
}

impl pbn::StepProvider<util::Timer> for MaxInfoGain {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        let mut min_expected_entropy = f64::INFINITY;
        for oidx in e.partition().keys() {
            if self.relevancy_prune {
                let force_step = pn::Step::SetClass(
                    *oidx,
                    pn::Class::True {
                        force_use: true,
                        assume: Some(true),
                    },
                    None,
                );
                match force_step.apply(e) {
                    None => continue,
                    Some(force_result) => {
                        if !pn::oracle::nonempty_completion(&force_result) {
                            continue;
                        }
                    }
                }
            }

            let mut steps = vec![];
            let mut entropy_sum = 0.0;
            for new_class in pn::Class::committed() {
                let step = pn::Step::SetClass(*oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(child) => {
                        match pn::oracle::log10_assume_model_count(&child) {
                            Some(h) => {
                                entropy_sum += h;
                                steps.push(step);
                            }
                            None => {}
                        }
                    }
                }
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

pub struct MinLeafHeuristic;

impl MinLeafHeuristic {
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

pub struct ForcedAssumptions;

impl ForcedAssumptions {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider<util::Timer> for ForcedAssumptions {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut r = Remaining::new();
        let steps = r.provide(_timer, e)?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphabeticalMode {
    Unsound,
    Complete,
    Relevant,
}

pub struct Alphabetical {
    mode: AlphabeticalMode,
    fancy_prune: bool,
}

impl Alphabetical {
    pub fn new(mode: AlphabeticalMode) -> Self {
        Self {
            mode,
            fancy_prune: false,
        }
    }
}

impl pbn::StepProvider<util::Timer> for Alphabetical {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
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
                        if !pn::oracle::nonempty_completion(&force_result) {
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
                            pn::oracle::nonempty_completion(&result)
                        }
                    };

                    let false_step =
                        pn::Step::SetClass(oidx, pn::Class::False, None);
                    let false_step_ok = match false_step.apply(e) {
                        None => false,
                        Some(result) => {
                            pn::oracle::nonempty_completion(&result)
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
                            || pn::oracle::nonempty_completion(&result)
                        {
                            ret.push(step);
                        }
                    }
                }
            }

            return Ok(ret);
        }
        return Ok(vec![]);
    }
}

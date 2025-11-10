use crate::partition_navigation as pn;
use crate::pbn::{self, Step};
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;
use rand::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// Commit

pub struct Commit;

impl Commit {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider for Commit {
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

impl pbn::StepProvider for Remaining {
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

impl pbn::StepProvider for Random {
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

impl pbn::StepProvider for TopDownInversion {
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

impl pbn::StepProvider for BottomUpInversion {
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

impl pbn::StepProvider for Leaf {
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

pub struct MaxInfoGain;

impl MaxInfoGain {
    pub fn new() -> Self {
        Self
    }
}

impl pbn::StepProvider for MaxInfoGain {
    type Step = pn::Step;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &pn::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        let mut min_expected_entropy = f64::INFINITY;
        for oidx in e.partition().keys() {
            let mut steps = vec![];
            let mut entropy_sum = 0.0;
            for new_class in pn::Class::committed() {
                let step = pn::Step::SetClass(*oidx, *new_class, None);
                match step.apply(e) {
                    None => continue,
                    Some(child) => match pn::oracle::entropy(&child) {
                        Some(h) => {
                            entropy_sum += h;
                            steps.push(step);
                        }
                        None => {}
                    },
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

impl pbn::StepProvider for MinLeafHeuristic {
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

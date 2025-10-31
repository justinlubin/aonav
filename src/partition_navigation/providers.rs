use crate::ao;
use crate::partition_navigation as pn;
use crate::pbn::{self, Step};
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexSet;
use rand::prelude::*;

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
        for (oidx, class) in e.partition() {
            if *class != pn::Class::Unseen {
                continue;
            }
            for new_class in pn::Class::all() {
                if new_class == class {
                    continue;
                }
                let step = pn::Step::SetClass(*oidx, *new_class);
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
        let unseen = e.filter_class(|c| c == pn::Class::Unseen).set;
        if unseen.is_empty() {
            return Ok(vec![]);
        }

        let oidx = unseen[rand::rng().random_range(0..unseen.len())];

        let mut ret = vec![];

        for new_class in pn::Class::all() {
            if *new_class == pn::Class::Unseen {
                continue;
            }
            let step = pn::Step::SetClass(oidx, *new_class);
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

fn forced_frontier(e: &pn::Exp) -> ao::AndSet {
    todo!()
    // let mut set = IndexSet::new();
    // let g = e.graph();
    // let mut worklist = vec![g.goal()];
    // let mut frontier = IndexSet::new();

    // 'worklist_loop: while let Some(oidx) = worklist.pop() {
    //     let providers = g.providers(oidx);
    //     for aidx in providers {
    //         if g.premises(aidx)
    //             .find(|&premise_oidx| {
    //                 e.class(premise_oidx) == pn::Class::True { force_use: true }
    //             })
    //             .is_some()
    //         {
    //             worklist.extend(g.premises(aidx));
    //             continue 'worklist_loop;
    //         }
    //     }

    // }

    // ao::AndSet { set: frontier }
}

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

        let true_force_use = e
            .filter_class(|c| c == pn::Class::True { force_use: true })
            .set;

        for oidx in true_force_use {
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

                let explore_step =
                    pn::Step::sequence(unseen.iter().map(|prem_oidx| {
                        pn::Step::SetClass(
                            *prem_oidx,
                            pn::Class::True { force_use: true },
                        )
                    }))
                    .unwrap();

                if let Some(result) = explore_step.apply(e) {
                    if pn::oracle::nonempty_completion(&result) {
                        ret.push(explore_step);
                    }
                }

                let commit_step =
                    pn::Step::sequence(unseen.into_iter().map(|prem_oidx| {
                        pn::Step::SetClass(
                            prem_oidx,
                            pn::Class::Assume { force_use: true },
                        )
                    }))
                    .unwrap();

                if let Some(result) = commit_step.apply(e) {
                    if pn::oracle::nonempty_completion(&result) {
                        ret.push(commit_step);
                    }
                }
            }
        }

        Ok(ret)
    }
}

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
        for (oidx, class) in e.partition() {
            if *class != pn::Class::Unseen {
                continue;
            }
            for new_class in pn::Class::all() {
                if new_class == class {
                    continue;
                }
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

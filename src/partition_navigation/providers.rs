use crate::partition_navigation as pn;
use crate::pbn::{self, Step};
use crate::util::{EarlyCutoff, Timer};

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

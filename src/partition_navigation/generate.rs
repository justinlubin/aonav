use crate::partition_navigation::*;

use rand::seq::SliceRandom;

fn minimize_one_step(e: &Exp) -> Option<Exp> {
    let mut part: Vec<_> = e.partition().iter().collect();
    part.shuffle(&mut rand::rng());
    for (oidx, original_class) in part {
        match *original_class {
            Class::True {
                assume: Some(true), ..
            } => {
                let mut ret = e.clone();
                let mut options = vec![
                    Class::Unknown,
                    Class::False,
                    Class::True {
                        force_use: false,
                        assume: Some(false),
                    },
                    Class::True {
                        force_use: true,
                        assume: Some(false),
                    },
                ];
                options.shuffle(&mut rand::rng());
                for new_class in options {
                    ret.unsafe_set_class(*oidx, new_class);
                    if oracle::valid(&ret) {
                        return Some(ret);
                    }
                }
            }
            _ => (),
        };
    }
    None
}

pub fn assumption_minimized(e: &Exp) -> Exp {
    let mut ret = e.clone();
    while let Some(m) = minimize_one_step(&ret) {
        ret = m;
    }
    return ret;
}

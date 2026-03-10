//! # SAT Solving
//!
//! Finds SAT assignments with a minimal number of literals let to true

// HELP! header?

use rustsat::{
    instances::SatInstance,
    solvers::{Solve, SolverResult},
    types::{constraints::CardConstraint, Assignment, Lit, TernaryVal},
};
use std::collections::HashSet;

// TODO make incremental
pub fn solve(
    instance: SatInstance,
    lits_to_minimize: &HashSet<Lit>,
) -> Option<Assignment> {
    let mut original_solver = rustsat_cadical::CaDiCaL::default();
    original_solver.add_cnf_ref(instance.cnf()).unwrap();

    if original_solver.solve().unwrap() != SolverResult::Sat {
        return None;
    }

    let original_sol = original_solver.full_solution().unwrap();

    let upper_bound: usize = lits_to_minimize
        .iter()
        .map(|lit| {
            if original_sol.lit_value(*lit) == TernaryVal::True {
                1
            } else {
                0
            }
        })
        .sum();

    let mut sol = original_sol;

    for k in 0..upper_bound {
        let mut new_instance = instance.clone();
        new_instance.add_card_constr(CardConstraint::new_ub(
            lits_to_minimize.iter().copied(),
            k,
        ));
        let mut solver = rustsat_cadical::CaDiCaL::default();
        solver.add_cnf_ref(instance.cnf()).unwrap();
        if solver.solve().unwrap() == SolverResult::Sat {
            sol = solver.full_solution().unwrap();
            break;
        }
    }

    Some(sol)
}

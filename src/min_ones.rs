use rustsat::{
    // algs::maxsat,
    // encodings::pb,
    // instances::{Objective, OptInstance},
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
    // let mut objective = Objective::new();
    // for lit in lits_to_minimize {
    //     objective.add_soft_lit(1, *lit);
    // }
    // type Alg = maxsat::SolutionImprovingSearch<
    //     rustsat_batsat::BasicSolver,
    //     pb::GeneralizedTotalizer,
    // >;
    // let opt_instance = OptInstance::compose(instance, objective);
    // opt_instance.solve_maxsat::<Alg>().map(|x| x.0)
    // solver.add_cnf(instance.).unwrap();
    // if solver.solve().unwrap() != SolverResult::Sat {
    //     return None;
    // }

    let mut original_solver = rustsat_batsat::BasicSolver::default();
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
        let mut solver = rustsat_batsat::BasicSolver::default();
        solver.add_cnf_ref(instance.cnf()).unwrap();
        if solver.solve().unwrap() == SolverResult::Sat {
            sol = solver.full_solution().unwrap();
            break;
        }
    }

    Some(sol)
}

use crate::partition_navigation::*;

use rustsat::instances::{Cnf, SatInstance};
use rustsat::solvers::Solve;
use rustsat::types::{constraints::CardConstraint, Lit};
use std::collections::HashMap;

fn add_lit_eq_cube(instance: &mut SatInstance, a: Lit, b: &[Lit]) {
    instance.add_lit_impl_cube(a, b);
    instance.add_cube_impl_lit(b, a);
}

fn add_lit_eq_clause(instance: &mut SatInstance, a: Lit, b: &[Lit]) {
    instance.add_lit_impl_clause(a, b);
    instance.add_clause_impl_lit(b, a);
}

fn compile(e: &Exp) -> Cnf {
    let graph = e.graph();

    let mut instance: SatInstance = SatInstance::new();

    // OR node truth values
    let mut o_true = HashMap::new();

    // AND node truth values
    let mut a_true = HashMap::new();

    // OR node on chosen derivation tree
    let mut o_active = HashMap::new();

    // AND node on chosen derivation tree
    let mut a_active = HashMap::new();

    for oidx in graph.or_indexes() {
        o_true.insert(oidx, instance.new_lit());
        o_active.insert(oidx, instance.new_lit());
    }

    for aidx in graph.and_indexes() {
        a_true.insert(aidx, instance.new_lit());
        a_active.insert(aidx, instance.new_lit());
    }

    // Add OR node constraints (semantic, activity, and consistency)

    for oidx in graph.or_indexes() {
        let is_true = *o_true.get(&oidx).unwrap();
        let is_active = *o_active.get(&oidx).unwrap();
        let class = e.class(oidx);

        // Add semantic and activity constraints

        // Activity: Active implies true
        instance.add_lit_impl_lit(is_active, is_true);

        let assume_semantics = match class {
            Class::Assume { .. } => true,
            _ => false,
        };

        if assume_semantics {
            // --- Semantic ---

            // Assume true
            instance.add_unit(is_true);

            // --- Activity ---

            // Providers cannot be active
            for aidx in graph.providers(oidx) {
                instance.add_unit(!*a_active.get(&aidx).unwrap());
            }
        } else {
            // --- Semantic ---

            // OR node true iff at least one provider true
            add_lit_eq_clause(
                &mut instance,
                is_true,
                &graph
                    .providers(oidx)
                    .map(|a| *a_true.get(&a).unwrap())
                    .collect::<Vec<_>>()[..],
            );

            // --- Activity ---

            let providers_active = graph
                .providers(oidx)
                .map(|a| *a_active.get(&a).unwrap())
                .collect::<Vec<_>>();

            // If at least one provider is active, then OR node is active
            instance.add_clause_impl_lit(&providers_active[..], is_active);

            // If OR node is active, then at least one provider is active
            instance.add_lit_impl_clause(is_active, &providers_active[..]);

            // At most one provider is active
            instance.add_card_constr(CardConstraint::new_eq(
                providers_active.into_iter(),
                1,
            ));
        }

        // Add consistency constraints

        match class {
            Class::Unseen => (),
            Class::Unknown => (),
            Class::False => instance.add_unit(!is_true),
            Class::True { force_use } => {
                instance.add_unit(is_true);
                if force_use {
                    instance.add_unit(is_active);
                }
            }
            Class::Assume { force_use } => {
                if force_use {
                    instance.add_unit(is_active);
                }
            }
        }
    }

    // Add AND node constraints (semantic, activity)

    for aidx in graph.and_indexes() {
        let is_true = *a_true.get(&aidx).unwrap();
        let is_active = *a_active.get(&aidx).unwrap();

        // --- Semantic ---

        // AND node true iff premises true
        add_lit_eq_cube(
            &mut instance,
            is_true,
            &graph
                .premises(aidx)
                .map(|o| *o_true.get(&o).unwrap())
                .collect::<Vec<_>>()[..],
        );

        // --- Activity ---

        // Active implies true
        instance.add_lit_impl_lit(is_active, is_true);

        let premises_active = graph
            .premises(aidx)
            .map(|o| *o_active.get(&o).unwrap())
            .collect::<Vec<_>>();

        // If at least one premise is active, then AND node must be active
        instance.add_clause_impl_lit(&premises_active[..], is_active);

        // if AND node is active, then all premises active
        instance.add_lit_impl_cube(is_active, &premises_active[..]);
    }

    instance.into_cnf().0
}

#[allow(dead_code)]
pub fn nonempty_completion(e: &Exp) -> bool {
    let cnf = compile(e);

    let mut solver = rustsat_batsat::BasicSolver::default();
    solver.add_cnf(cnf).unwrap();

    solver.solve().unwrap() == rustsat::solvers::SolverResult::Sat
}

pub fn main() {
    let mut instance: SatInstance = SatInstance::new();
    let l1 = instance.new_lit();
    let l2 = instance.new_lit();
    instance.add_binary(l1, l2);
    instance.add_binary(!l1, l2);
    instance.add_unit(l1);
    let mut solver = rustsat_batsat::BasicSolver::default();
    solver.add_cnf(instance.into_cnf().0).unwrap();
    let res = solver.solve().unwrap();
    let sol = solver.full_solution().unwrap();
    println!("{:?}", res);
    println!("{:?}", sol[l1.var()]);
    println!("{:?}", sol[l2.var()]);
    std::process::exit(1);
}

// use varisat::solver::Solver;
// use varisat::{CnfFormula, ExtendFormula};
//
// #[allow(dead_code)]
// pub fn nonempty_completion(_e: Exp) -> bool {
//     todo!()
// }
//
// #[derive(Debug, Clone)]
// enum Formula {
//     Var(varisat::Var),
//     And(Vec<Formula>),
//     Or(Vec<Formula>),
//     Not(Box<Formula>),
// }
//
// impl Formula {
//     fn eq(f1: Self, f2: Self) -> Self {
//         Formula::Or(vec![
//             Formula::And(vec![f1.clone(), f2.clone()]),
//             Formula::And(vec![
//                 Formula::Not(Box::new(f1)),
//                 Formula::Not(Box::new(f2)),
//             ]),
//         ])
//     }
//
//     fn implies(f1: Self, f2: Self) -> Self {
//         Formula::Or(vec![Formula::Not(Box::new(f1)), f2])
//     }
//
//     pub fn cnf(&self) -> &[varisat::Lit] {}
// }
//
// // "A == B" is (A + B') * (A' + B)
// fn eq() -> CnfFormula {}
//
// pub fn main() {
//     let mut solver = Solver::new();
//     let (x, y, z) = solver.new_lits();
//     solver.add_clause(&[x, y]);
//     solver.add_clause(&[!x]);
//     solver.add_clause(&[!y]);
//     let solution = solver.solve();
//     println!("{:?}", solution);
//     println!("{:?}", solver.model());
//     std::process::exit(1);
// }

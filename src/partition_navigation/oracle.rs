use crate::ao;
use crate::partition_navigation::*;

use rustsat::instances::SatInstance;
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

struct Vars {
    o_true: HashMap<ao::OIdx, Lit>,
    a_true: HashMap<ao::AIdx, Lit>,
    o_active: HashMap<ao::OIdx, Lit>,
    a_active: HashMap<ao::AIdx, Lit>,
}

fn make_vars(instance: &mut SatInstance, e: &Exp) -> Vars {
    // OR node truth values
    let mut o_true = HashMap::new();

    // AND node truth values
    let mut a_true = HashMap::new();

    // OR node on chosen derivation tree
    let mut o_active = HashMap::new();

    // AND node on chosen derivation tree
    let mut a_active = HashMap::new();

    for oidx in e.graph().or_indexes() {
        o_true.insert(oidx, instance.new_lit());
        o_active.insert(oidx, instance.new_lit());
    }

    for aidx in e.graph().and_indexes() {
        a_true.insert(aidx, instance.new_lit());
        a_active.insert(aidx, instance.new_lit());
    }

    Vars {
        o_true,
        a_true,
        o_active,
        a_active,
    }
}

fn compile(instance: &mut SatInstance, vars: &Vars, e: &Exp) {
    let graph = e.graph();
    let Vars {
        o_true,
        a_true,
        o_active,
        a_active,
    } = vars;

    // Add OR node constraints (semantic, activity, and consistency)

    for oidx in graph.or_indexes() {
        let is_true = *o_true.get(&oidx).unwrap();
        let is_active = *o_active.get(&oidx).unwrap();
        let class = e.class(oidx);

        // Add semantic and activity constraints

        // --- Activity (universal) ---
        instance.add_lit_impl_lit(is_active, is_true);

        // If non-goal OR node active, then at least one consumer is active
        if oidx != graph.goal() {
            let consumers_active = graph
                .consumers(oidx)
                .map(|a| *a_active.get(&a).unwrap())
                .collect::<Vec<_>>();

            instance.add_lit_impl_clause(is_active, &consumers_active[..]);
        }

        // Optimistically assume all unseen nodes are assumed
        let assume_semantics = match class {
            Class::Assume { .. } | Class::Unseen => true,
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
                instance,
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
            instance,
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

        // if AND node is active, then all premises active
        instance.add_lit_impl_cube(is_active, &premises_active[..]);
    }
}

#[allow(dead_code)]
pub fn nonempty_completion(e: &Exp) -> bool {
    let mut instance = SatInstance::new();

    let vars = make_vars(&mut instance, e);

    compile(&mut instance, &vars, e);
    let cnf = instance.into_cnf().0;

    let mut solver = rustsat_batsat::BasicSolver::default();
    solver.add_cnf(cnf).unwrap();

    let ok = solver.solve().unwrap() == rustsat::solvers::SolverResult::Sat;

    if ok {
        println!("{}", e);
        let sol = solver.full_solution().unwrap();
        for (oidx, lit) in vars.o_true {
            println!("{} true: {}", e.graph().or_at(oidx), sol[lit.var()])
        }
        for (oidx, lit) in vars.o_active {
            println!("{} active: {}", e.graph().or_at(oidx), sol[lit.var()])
        }
        for (aidx, lit) in vars.a_true {
            println!("{} true: {}", e.graph().and_at(aidx), sol[lit.var()])
        }
        for (aidx, lit) in vars.a_active {
            println!("{} active: {}", e.graph().and_at(aidx), sol[lit.var()])
        }
    }

    ok
}

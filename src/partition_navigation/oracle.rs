use crate::ao;
use crate::min_ones;
use crate::model_count;
use crate::partition_navigation::*;
use crate::pbn;

use rustsat::instances::ManageVars;
use rustsat::instances::SatInstance;
use rustsat::solvers::Solve;
use rustsat::types::{constraints::CardConstraint, Lit};
use std::collections::HashMap;
use std::collections::HashSet;

////////////////////////////////////////////////////////////////////////////////
// Main oracle implementation

// Helpers

fn add_lit_eq_cube(instance: &mut SatInstance, a: Lit, b: &[Lit]) {
    instance.add_lit_impl_cube(a, b);
    instance.add_cube_impl_lit(b, a);
}

fn add_guarded_lit_eq_clause(
    instance: &mut SatInstance,
    guard: Lit,
    a: Lit,
    b: &[Lit],
) {
    instance.add_cube_impl_clause(&[guard, a], b);
    for bi in b {
        instance.add_cube_impl_lit(&[guard, *bi], a);
    }
}

// Compilation

struct Context<'a> {
    instance: SatInstance,
    exp: &'a Exp,
    // Whether or not OR node should use assume semantics
    o_assume: HashMap<ao::OIdx, Lit>,
    // OR node truth values
    o_true: HashMap<ao::OIdx, Lit>,
    // OR node on chosen derivation tree
    o_active: HashMap<ao::OIdx, Lit>,
    // AND node truth values
    a_true: HashMap<ao::AIdx, Lit>,
    // AND node on chosen derivation tree
    a_active: HashMap<ao::AIdx, Lit>,
}

impl<'a> Context<'a> {
    fn compile(instance: SatInstance, exp: &'a Exp) -> Self {
        let mut ret = Self {
            instance,
            exp,
            o_assume: HashMap::new(),
            o_true: HashMap::new(),
            o_active: HashMap::new(),
            a_true: HashMap::new(),
            a_active: HashMap::new(),
        };

        for oidx in exp.graph().or_indexes() {
            ret.o_assume.insert(oidx, ret.instance.new_lit());
            ret.o_true.insert(oidx, ret.instance.new_lit());
            ret.o_active.insert(oidx, ret.instance.new_lit());
        }

        for aidx in exp.graph().and_indexes() {
            ret.a_true.insert(aidx, ret.instance.new_lit());
            ret.a_active.insert(aidx, ret.instance.new_lit());
        }

        ret.add_constraints();

        ret
    }

    fn add_constraints(&mut self) {
        for oidx in self.exp.graph().or_indexes() {
            self.or(oidx)
        }

        for aidx in self.exp.graph().and_indexes() {
            self.and(aidx)
        }
    }

    fn or_semantics(&mut self, oidx: ao::OIdx) {
        let graph = self.exp.graph();

        let is_assume = *self.o_assume.get(&oidx).unwrap();
        let is_true = *self.o_true.get(&oidx).unwrap();

        // Non-assume OR node true iff a provider is true
        add_guarded_lit_eq_clause(
            &mut self.instance,
            !is_assume,
            is_true,
            &graph
                .providers(oidx)
                .map(|a| *self.a_true.get(&a).unwrap())
                .collect::<Vec<_>>()[..],
        );

        // Assume OR node unconditionally true
        self.instance.add_lit_impl_lit(is_assume, is_true);
    }

    fn or_activity(&mut self, oidx: ao::OIdx) {
        let graph = self.exp.graph();

        let is_assume = *self.o_assume.get(&oidx).unwrap();
        let is_true = *self.o_true.get(&oidx).unwrap();
        let is_active = *self.o_active.get(&oidx).unwrap();

        // Active implies true
        self.instance.add_lit_impl_lit(is_active, is_true);

        if oidx != graph.goal() {
            let consumers_active = graph
                .consumers(oidx)
                .map(|a| *self.a_active.get(&a).unwrap())
                .collect::<Vec<_>>();

            // Active non-goal nodes must have active consumer
            self.instance
                .add_lit_impl_clause(is_active, &consumers_active[..]);
        }

        let providers_active = graph
            .providers(oidx)
            .map(|a| *self.a_active.get(&a).unwrap())
            .collect::<Vec<_>>();

        // Providers of assume nodes cannot be active
        self.instance.add_lit_impl_cube(
            is_assume,
            &providers_active.iter().map(|x| !*x).collect::<Vec<_>>()[..],
        );

        // Non-assume OR node active iff at least one provider active
        add_guarded_lit_eq_clause(
            &mut self.instance,
            !is_assume,
            is_active,
            &providers_active[..],
        );

        // Unconditionally true that at most one provider is active
        self.instance.add_card_constr(CardConstraint::new_ub(
            providers_active.into_iter(),
            1,
        ));
    }

    fn or_consistency(&mut self, oidx: ao::OIdx) {
        let class = self.exp.class(oidx);

        let is_assume = *self.o_assume.get(&oidx).unwrap();
        let is_true = *self.o_true.get(&oidx).unwrap();
        let is_active = *self.o_active.get(&oidx).unwrap();

        match class {
            Class::Unseen => (),
            Class::Unknown => self.instance.add_unit(!is_assume),
            Class::False => {
                self.instance.add_unit(!is_assume);
                self.instance.add_unit(!is_true);
                self.instance.add_unit(!is_active);
            }
            Class::True { force_use, assume } => {
                self.instance.add_unit(is_true);
                if force_use {
                    self.instance.add_unit(is_active);
                }
                match assume {
                    Some(false) => self.instance.add_unit(!is_assume),
                    Some(true) => self.instance.add_unit(is_assume),
                    None => (),
                }
            }
        }
    }

    fn or(&mut self, oidx: ao::OIdx) {
        self.or_semantics(oidx);
        self.or_activity(oidx);
        self.or_consistency(oidx);
    }

    fn and_semantics(&mut self, aidx: ao::AIdx) {
        let is_true = *self.a_true.get(&aidx).unwrap();

        add_lit_eq_cube(
            &mut self.instance,
            is_true,
            &self
                .exp
                .graph()
                .premises(aidx)
                .map(|o| *self.o_true.get(&o).unwrap())
                .collect::<Vec<_>>()[..],
        );
    }

    fn and_activity(&mut self, aidx: ao::AIdx) {
        let graph = self.exp.graph();

        let is_true = *self.a_true.get(&aidx).unwrap();
        let is_active = *self.a_active.get(&aidx).unwrap();

        // Active implies true
        self.instance.add_lit_impl_lit(is_active, is_true);

        let premises_active = graph
            .premises(aidx)
            .map(|o| *self.o_active.get(&o).unwrap())
            .collect::<Vec<_>>();

        // AND node is active implies all premises active
        self.instance
            .add_lit_impl_cube(is_active, &premises_active[..]);
    }

    fn and(&mut self, aidx: ao::AIdx) {
        self.and_semantics(aidx);
        self.and_activity(aidx);
    }
}

#[allow(dead_code)]
pub fn nonempty_completion(e: &Exp) -> bool {
    let ctx = Context::compile(SatInstance::new(), e);
    let cnf = ctx.instance.into_cnf().0;

    let mut solver = rustsat_batsat::BasicSolver::default();
    solver.add_cnf(cnf).unwrap();

    let ok = solver.solve().unwrap() == rustsat::solvers::SolverResult::Sat;

    if ok && log::log_enabled!(log::Level::Debug) {
        log::debug!("{}", e);
        let sol = solver.full_solution().unwrap();
        for (oidx, lit) in ctx.o_true {
            log::debug!("{} true: {}", e.graph().or_at(oidx), sol[lit.var()])
        }
        for (oidx, lit) in ctx.o_active {
            log::debug!("{} active: {}", e.graph().or_at(oidx), sol[lit.var()])
        }
        for (aidx, lit) in ctx.a_true {
            log::debug!("{} true: {}", e.graph().and_at(aidx), sol[lit.var()])
        }
        for (aidx, lit) in ctx.a_active {
            log::debug!("{} active: {}", e.graph().and_at(aidx), sol[lit.var()])
        }
    }

    ok
}

////////////////////////////////////////////////////////////////////////////////
// Entropy

pub fn entropy(e: &Exp) -> Option<f64> {
    let ctx = Context::compile(SatInstance::new(), e);
    let (cnf, vm) = ctx.instance.into_cnf();
    model_count::log10_model_count(
        vm.n_used(),
        &cnf,
        Some(ctx.o_assume.values().map(|lit| lit.var()).collect()),
    )
}

////////////////////////////////////////////////////////////////////////////////
// Minimal leaves

pub fn minimal_leaves(e: &Exp) -> Option<ao::OrSet> {
    let mut e = e.clone();

    let unseen_leaves: HashSet<_> = e
        .graph()
        .or_leaves()
        .filter(|oidx| e.class(*oidx) == Class::Unseen)
        .collect();

    e.set_remaining_pessimistically(&unseen_leaves);

    let ctx = Context::compile(SatInstance::new(), &e);

    let leaf_map: HashMap<_, _> = unseen_leaves
        .into_iter()
        .map(|oidx| (oidx, *ctx.o_assume.get(&oidx).unwrap()))
        .collect();

    let assignment =
        min_ones::solve(ctx.instance, &leaf_map.values().copied().collect())?;

    Some(ao::OrSet {
        set: leaf_map
            .into_iter()
            .filter_map(|(oidx, lit)| {
                if assignment.lit_value(lit) == rustsat::types::TernaryVal::True
                {
                    Some(oidx)
                } else {
                    None
                }
            })
            .collect(),
    })
}

////////////////////////////////////////////////////////////////////////////////
// Validity checker

pub fn valid(e: &Exp) -> bool {
    let mut e = e.clone();
    e.set_remaining_pessimistically(&HashSet::new());
    nonempty_completion(&e)
}

pub struct Valid;

impl Valid {
    pub fn new() -> Self {
        Self {}
    }
}

impl pbn::ValidityChecker for Valid {
    type Exp = Exp;

    fn check(&self, e: &Self::Exp) -> bool {
        valid(e)
    }
}

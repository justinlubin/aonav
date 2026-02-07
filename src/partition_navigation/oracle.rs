use crate::min_ones;
use crate::model_count;
use crate::partition_navigation::*;

use aograph as ao;
use indexmap::IndexMap;
use rustsat::instances::ManageVars;
use rustsat::instances::SatInstance;
use rustsat::solvers::{Solve, SolveIncremental};
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

fn add_cube(instance: &mut SatInstance, a: &[Lit]) {
    for ai in a {
        instance.add_unit(*ai);
    }
}

// Compilation

struct CompileContext {
    graph: ao::Graph,
    instance: SatInstance,
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

impl CompileContext {
    fn compile(exp: &Exp) -> Self {
        let mut ret = Self::compile_shared(exp.graph().clone());
        let assumptions = ret.or_consistency_lits(exp.partition());
        add_cube(&mut ret.instance, &assumptions);
        ret
    }

    fn compile_shared(graph: ao::Graph) -> Self {
        let mut ret = Self {
            graph,
            instance: SatInstance::new(),
            o_assume: HashMap::new(),
            o_true: HashMap::new(),
            o_active: HashMap::new(),
            a_true: HashMap::new(),
            a_active: HashMap::new(),
        };

        for oidx in ret.graph.or_indexes() {
            ret.o_assume.insert(oidx, ret.instance.new_lit());
            ret.o_true.insert(oidx, ret.instance.new_lit());
            ret.o_active.insert(oidx, ret.instance.new_lit());
        }

        for aidx in ret.graph.and_indexes() {
            ret.a_true.insert(aidx, ret.instance.new_lit());
            ret.a_active.insert(aidx, ret.instance.new_lit());
        }

        ret.add_shared_constraints();

        ret
    }

    fn add_shared_constraints(&mut self) {
        for oidx in self.graph.or_indexes().collect::<Vec<_>>() {
            self.shared_or(oidx)
        }

        for aidx in self.graph.and_indexes().collect::<Vec<_>>() {
            self.shared_and(aidx)
        }
    }

    fn or_semantics(&mut self, oidx: ao::OIdx) {
        let is_assume = *self.o_assume.get(&oidx).unwrap();
        let is_true = *self.o_true.get(&oidx).unwrap();

        // Non-assume OR node true iff a provider is true
        add_guarded_lit_eq_clause(
            &mut self.instance,
            !is_assume,
            is_true,
            &self
                .graph
                .providers(oidx)
                .map(|a| *self.a_true.get(&a).unwrap())
                .collect::<Vec<_>>()[..],
        );

        // Assume OR node unconditionally true
        self.instance.add_lit_impl_lit(is_assume, is_true);
    }

    fn or_activity(&mut self, oidx: ao::OIdx) {
        let is_assume = *self.o_assume.get(&oidx).unwrap();
        let is_true = *self.o_true.get(&oidx).unwrap();
        let is_active = *self.o_active.get(&oidx).unwrap();

        // Active implies true
        self.instance.add_lit_impl_lit(is_active, is_true);

        if oidx != self.graph.goal() {
            let consumers_active = self
                .graph
                .consumers(oidx)
                .map(|a| *self.a_active.get(&a).unwrap())
                .collect::<Vec<_>>();

            // Active non-goal nodes must have active consumer
            self.instance
                .add_lit_impl_clause(is_active, &consumers_active[..]);
        }

        let providers_active = self
            .graph
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

    fn or_consistency_lits(
        &mut self,
        partition: &IndexMap<ao::OIdx, Class>,
    ) -> Vec<Lit> {
        let mut ret = vec![];

        for (oidx, class) in partition {
            let is_assume = *self.o_assume.get(&oidx).unwrap();
            let is_true = *self.o_true.get(&oidx).unwrap();
            let is_active = *self.o_active.get(&oidx).unwrap();

            match class {
                Class::Unseen => (),
                Class::Unknown => ret.push(!is_assume),
                Class::False => {
                    ret.push(!is_assume);
                    ret.push(!is_true);
                    ret.push(!is_active);
                }
                Class::True { force_use, assume } => {
                    ret.push(is_true);
                    if *force_use {
                        ret.push(is_active);
                    }
                    match assume {
                        Some(false) => ret.push(!is_assume),
                        Some(true) => ret.push(is_assume),
                        None => (),
                    }
                }
            }
        }

        ret
    }

    fn shared_or(&mut self, oidx: ao::OIdx) {
        self.or_semantics(oidx);
        self.or_activity(oidx);
    }

    fn and_semantics(&mut self, aidx: ao::AIdx) {
        let is_true = *self.a_true.get(&aidx).unwrap();

        add_lit_eq_cube(
            &mut self.instance,
            is_true,
            &self
                .graph
                .premises(aidx)
                .map(|o| *self.o_true.get(&o).unwrap())
                .collect::<Vec<_>>()[..],
        );
    }

    fn and_activity(&mut self, aidx: ao::AIdx) {
        let is_true = *self.a_true.get(&aidx).unwrap();
        let is_active = *self.a_active.get(&aidx).unwrap();

        // Active implies true
        self.instance.add_lit_impl_lit(is_active, is_true);

        let premises_active = self
            .graph
            .premises(aidx)
            .map(|o| *self.o_active.get(&o).unwrap())
            .collect::<Vec<_>>();

        // AND node is active implies all premises active
        self.instance
            .add_lit_impl_cube(is_active, &premises_active[..]);
    }

    fn shared_and(&mut self, aidx: ao::AIdx) {
        self.and_semantics(aidx);
        self.and_activity(aidx);
    }
}

pub struct IncrementalContext {
    solver: rustsat_cadical::CaDiCaL<'static, 'static>,
    ctx: CompileContext,
}

impl IncrementalContext {
    pub fn new(e: &Exp) -> Self {
        let mut ctx = CompileContext::compile_shared(e.graph().clone());
        let instance = std::mem::take(&mut ctx.instance);
        let cnf = instance.into_cnf().0;

        let mut solver = rustsat_cadical::CaDiCaL::default();
        solver.add_cnf(cnf).unwrap();

        Self { solver, ctx }
    }

    pub fn nonempty_completion(&mut self, e: &Exp) -> bool {
        let assumptions = self.ctx.or_consistency_lits(e.partition());

        let ok = self.solver.solve_assumps(&assumptions[..]).unwrap()
            == rustsat::solvers::SolverResult::Sat;

        if ok && log::log_enabled!(log::Level::Debug) {
            log::debug!("{}", e);
            let sol = self.solver.full_solution().unwrap();
            for (oidx, lit) in &self.ctx.o_true {
                log::debug!(
                    "{} true: {}",
                    e.graph().or_at(*oidx),
                    sol[lit.var()]
                )
            }
            for (oidx, lit) in &self.ctx.o_active {
                log::debug!(
                    "{} active: {}",
                    e.graph().or_at(*oidx),
                    sol[lit.var()]
                )
            }
            for (aidx, lit) in &self.ctx.a_true {
                log::debug!(
                    "{} true: {}",
                    e.graph().and_at(*aidx),
                    sol[lit.var()]
                )
            }
            for (aidx, lit) in &self.ctx.a_active {
                log::debug!(
                    "{} active: {}",
                    e.graph().and_at(*aidx),
                    sol[lit.var()]
                )
            }
        }

        ok
    }
}

pub enum OptInc {
    Incremental(IncrementalContext),
    NonIncremental,
}

impl OptInc {
    pub fn from_optional_start(start: Option<&Exp>) -> Self {
        match start {
            Some(e) => Self::Incremental(IncrementalContext::new(e)),
            None => Self::NonIncremental,
        }
    }

    pub fn nonempty_completion(&mut self, e: &Exp) -> bool {
        match self {
            Self::Incremental(inc) => inc.nonempty_completion(e),
            Self::NonIncremental => {
                IncrementalContext::new(e).nonempty_completion(e)
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Entropy

pub fn log10_assume_model_count(e: &Exp, projected: &ao::OrSet) -> Option<f64> {
    let ctx = CompileContext::compile(e);
    let (cnf, vm) = ctx.instance.into_cnf();

    model_count::log10_model_count(
        vm.n_used(),
        &cnf,
        Some(
            ctx.o_assume
                .iter()
                .filter_map(|(oidx, lit)| {
                    if projected.set.contains(oidx) {
                        Some(lit.var())
                    } else {
                        None
                    }
                })
                .collect(),
        ),
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

    let ctx = CompileContext::compile(&e);

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

pub struct Valid {
    incremental: OptInc,
}

impl Valid {
    pub fn new(incremental: OptInc) -> Self {
        Self { incremental }
    }
}

impl pbn::ValidityChecker for Valid {
    type Exp = Exp;

    fn check(&mut self, e: &Self::Exp) -> bool {
        let mut e = e.clone();
        e.set_remaining_pessimistically(&HashSet::new());
        self.incremental.nonempty_completion(&e)
    }
}

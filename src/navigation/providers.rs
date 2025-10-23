use crate::navigation::*;
use crate::pbn;
use crate::util::{EarlyCutoff, Timer};

use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// Committal add provider

pub enum CommittalAddAlgorithm {
    Naive,
}

pub struct CommittalAddProvider<A, O> {
    phantom: PhantomData<(A, O)>,
    #[allow(dead_code)]
    algorithm: CommittalAddAlgorithm,
}

impl<A: Clone, O: Clone> CommittalAddProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            algorithm: CommittalAddAlgorithm::Naive,
        }
    }
}

// TODO make compatible with "remove" steps
impl<A: Clone, O: Clone> pbn::StepProvider for CommittalAddProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut next_labels = IndexSet::new();

        let committed = e.committed();
        for axs in ao::algo::proper_axiom_sets(e.graph(), e.graph().goal()) {
            if committed.set.is_subset(&axs.set) {
                next_labels.extend(axs.set.difference(&committed.set).cloned())
            }
        }

        let steps: Vec<_> = next_labels
            .into_iter()
            .map(|o| {
                Step::SetClass(
                    o,
                    PartitionClass::ShouldBeTrue {
                        will_provide: true,
                        force_use: true,
                    },
                    PhantomData,
                )
            })
            .collect();

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Complete refine provider

pub struct CompleteRefineProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> CompleteRefineProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for CompleteRefineProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        _e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let steps = vec![];

        // for x in e.committed().set {
        //     for axs in ao::algo::proper_axiom_sets(e.graph(), x) {
        //         if axs.set == IndexSet::from([x]) {
        //             continue;
        //         }
        //         steps.push(Step::Refine(x, axs))
        //     }
        // }

        Ok(steps)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Arbitrary subset commit provider

pub struct ArbitrarySubsetCommitProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> ArbitrarySubsetCommitProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider
    for ArbitrarySubsetCommitProvider<A, O>
{
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        if valid(e) {
            return Ok(vec![]);
        }

        let mut subset = e.provided();

        if !consistent_with(e, &subset, &ao::NodeSet::new()) {
            return Ok(vec![]);
        }

        'fixpoint: loop {
            for x in &subset.set {
                let c = e.graph.or_data_ref(*x).unwrap().class;
                if c.provided() && !c.committed() {
                    let mut candidate = subset.clone();
                    candidate.set.swap_remove(x);
                    if consistent_with(e, &candidate, &ao::NodeSet::new()) {
                        subset = candidate;
                        continue 'fixpoint;
                    }
                }
            }
            break;
        }

        let step = match Step::sequence(subset.set.into_iter().map(|oid| {
            Step::SetClass(
                oid,
                PartitionClass::ShouldBeTrue {
                    will_provide: true,
                    force_use: true,
                },
                PhantomData,
            )
        })) {
            Some(x) => x,
            None => return Ok(vec![]),
        };

        Ok(vec![step])
    }
}

////////////////////////////////////////////////////////////////////////////////
// Random provider

pub struct RandomProvider<A, O> {
    phantom: PhantomData<(A, O)>,
}

impl<A, O> RandomProvider<A, O> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<A: Clone, O: Clone> pbn::StepProvider for RandomProvider<A, O> {
    type Step = Step<A, O>;

    fn provide(
        &mut self,
        _timer: &Timer,
        e: &Exp<A, O>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut rest = e.unseen().set;
        if rest.is_empty() {
            return Ok(vec![]);
        }
        let oid = rest[rand::rng().random_range(0..rest.len())];
        let _ = rest.swap_remove(&oid);

        let mut ret = vec![Step::SetClass(
            oid,
            PartitionClass::ShouldBeTrue {
                will_provide: true,
                force_use: false,
            },
            PhantomData,
        )];

        if !consistent_with(
            e,
            &ao::NodeSet {
                set: e.committed().set.union(&rest).cloned().collect(),
            },
            &ao::NodeSet::new(),
        ) {
            return Ok(ret);
        }

        ret.push(Step::SetClass(
            oid,
            PartitionClass::ShouldBeFalse,
            PhantomData,
        ));

        ret.push(Step::SetClass(
            oid,
            PartitionClass::ShouldBeTrue {
                will_provide: false,
                force_use: false,
            },
            PhantomData,
        ));

        Ok(ret)
    }
}

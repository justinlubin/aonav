use crate::ao::*;
use indexmap::IndexSet;
use std::collections::HashMap;

// Graph operations

// Uses forward chaining
// Reference: https://courses.cs.washington.edu/courses/cse473/12au/slides/lect10.pdf
pub fn provable_or_nodes<A, O>(graph: &Graph<A, O>) -> NodeSet {
    let mut count: HashMap<AIdx, usize> = HashMap::new();
    let mut inferred: IndexSet<OIdx> = IndexSet::new();
    let mut agenda: Vec<OIdx> =
        graph.sources().map(|aid| graph.conclusion(aid)).collect();

    while let Some(p) = agenda.pop() {
        if !inferred.insert(p) {
            continue;
        }

        for c in graph.consumers(p) {
            *count.entry(c).or_insert_with(|| graph.premises(c).count()) -= 1;

            if count[&c] == 0 {
                agenda.push(graph.conclusion(c))
            }
        }
    }

    NodeSet { set: inferred }
}

// TODO switch to using backward reasoning
pub fn provable<A, O>(graph: &Graph<A, O>, oid: OIdx) -> bool {
    provable_or_nodes(graph).set.contains(&oid)
}

pub fn proper_axiom_sets<A: Clone, O: Clone>(
    graph: &Graph<A, O>,
) -> Vec<NodeSet> {
    let mut ret: Vec<NodeSet> = vec![];
    let mut agenda: Vec<NodeSet> = vec![NodeSet {
        set: IndexSet::new(),
    }];
    let or_indexes: Vec<OIdx> = graph.or_indexes().collect();

    while let Some(axs) = agenda.pop() {
        let mut ax_graph = graph.clone();
        ax_graph.make_axioms(axs.set.iter().cloned());
        if provable(&ax_graph, ax_graph.goal()) {
            let mut new_ret = vec![];
            let mut should_add = true;
            for ret_axs in ret {
                if ret_axs.set.is_subset(&axs.set) {
                    should_add = false;
                }
                if !axs.set.is_subset(&ret_axs.set) {
                    new_ret.push(ret_axs);
                }
            }
            if should_add {
                new_ret.push(axs);
            }
            ret = new_ret;
        } else {
            'label: for o in or_indexes.iter().cloned() {
                let mut new_axs = axs.clone();
                if new_axs.set.insert(o) {
                    for ret_axs in &ret {
                        if ret_axs.set.is_subset(&new_axs.set) {
                            continue 'label;
                        }
                    }
                    agenda.push(new_axs);
                };
            }
        }
    }

    if log::log_enabled!(log::Level::Debug) {
        let mut msg = "Proper axiom sets: [".to_owned();
        for ret_axs in &ret {
            msg += &format!(" {}", ret_axs.show(graph));
        }
        msg += " ]";
        log::debug!("{}", msg);
    }

    ret
}

pub fn provable_with<A: Clone, O: Clone>(
    graph: &Graph<A, O>,
    axioms: &NodeSet,
) -> bool {
    let mut ax_graph = graph.clone();
    ax_graph.make_axioms(axioms.set.iter().cloned());
    provable(&ax_graph, ax_graph.goal())
}

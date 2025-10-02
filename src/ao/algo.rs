use crate::ao::*;
use std::collections::{HashMap, HashSet};

impl<A, O> Graph<A, O> {
    // Graph operations

    // Uses forward chaining
    // Reference: https://courses.cs.washington.edu/courses/cse473/12au/slides/lect10.pdf
    pub fn provable_or_nodes(&self) -> HashSet<OIdx> {
        let mut count: HashMap<AIdx, usize> = HashMap::new();
        let mut inferred: HashSet<OIdx> = HashSet::new();
        let mut agenda: Vec<OIdx> =
            self.sources().map(|aid| self.conclusion(aid)).collect();

        while let Some(p) = agenda.pop() {
            if !inferred.insert(p) {
                continue;
            }

            for c in self.consumers(p) {
                *count.entry(c).or_insert_with(|| self.premises(c).count()) -=
                    1;

                if count[&c] == 0 {
                    agenda.push(self.conclusion(c))
                }
            }
        }

        inferred
    }

    // TODO switch to using backward reasoning
    pub fn provable_or_node(&self, oid: OIdx) -> bool {
        self.provable_or_nodes().contains(&oid)
    }

    // TODO Maybe simply switch to a pass over all AND nodes that removes
    // any with no head (after clearing out all ORs)? Seems not strictly
    // necessary to include the AND pass at all, though, just cosmetic?
    // pub fn reduce(&mut self) {
    //     for oid in self.provable_or_nodes() {
    //         for aid in self.or_preds(oid).collect::<Vec<_>>() {
    //             if self.and_succs(aid).count() == 1 {
    //                 let _ = self.pg.remove_node(aid.0);
    //             }
    //         }
    //         let _ = self.pg.remove_node(oid.0);
    //     }
    // }
}

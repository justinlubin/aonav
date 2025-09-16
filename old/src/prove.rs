use crate::core::*;
use crate::{transform, util};

pub fn top_down(rules: &ProofSystem, prop: &str) -> Vec<Proof> {
    transform::providing_rules(rules, prop)
        .into_iter()
        .flat_map(|r| {
            util::cartesian_product(
                r.premises
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| (i, top_down(rules, &p)))
                    .collect(),
            )
            .into_iter()
            .map(move |subproofs| Proof {
                premises: subproofs.into_iter().map(|(_, x)| x).collect(),
                conclusion: prop.to_string(),
                rule_name: r.name.clone(),
            })
        })
        .collect()
}

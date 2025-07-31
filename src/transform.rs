use crate::core::*;

use indexmap::{IndexMap, IndexSet};
use petgraph::Graph;

pub fn props(rules: &ProofSystem) -> IndexSet<String> {
    rules
        .iter()
        .flat_map(|r| {
            std::iter::once(r.conclusion.clone())
                .chain(r.premises.iter().cloned())
        })
        .collect()
}

pub fn providing_rules(rules: &ProofSystem, prop: &str) -> ProofSystem {
    rules
        .iter()
        .filter(|r| r.conclusion == prop)
        .cloned()
        .collect()
}

pub fn dualize(rules: &ProofSystem) -> ProofSystem {
    rules
        .iter()
        .flat_map(|r| {
            r.premises.iter().enumerate().map(|(i, p)| Rule {
                premises: vec![format!("un:{}", p)],
                conclusion: format!("na:{}", r.name),
                name: format!("na.{}.{}", r.name, i + 1),
            })
        })
        .chain(props(rules).into_iter().map(|p| {
            Rule {
                premises: providing_rules(rules, &p)
                    .into_iter()
                    .map(|r| format!("na:{}", r.name))
                    .collect(),
                conclusion: format!("un:{}", p),
                name: format!("un.{}", p),
            }
        }))
        .collect()
}

pub fn make_graph(ps: &ProofSystem) -> ProofSystemGraph {
    let prs = props(ps);
    let mut im = IndexMap::with_capacity(prs.len());
    let mut g = Graph::with_capacity(ps.len(), 3 * ps.len());
    for p in prs {
        let i = g.add_node(PSGNode::Prop(p.clone()));
        let _ = im.insert(p, i);
    }
    for r in ps {
        let i = g.add_node(PSGNode::Rule(r.name.clone()));
        let _ = g.add_edge(i, *im.get(&r.conclusion).unwrap(), PSGEdge::Or);
        for prem in &r.premises {
            let _ = g.add_edge(*im.get(prem).unwrap(), i, PSGEdge::And);
        }
    }
    g
}

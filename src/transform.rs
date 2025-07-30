use crate::core::*;

use indexmap::IndexSet;

pub fn props(rules: &Vec<Rule>) -> IndexSet<String> {
    rules
        .iter()
        .flat_map(|r| std::iter::once(r.conclusion.clone()).chain(r.premises.iter().cloned()))
        .collect()
}

pub fn providing_rules(rules: &Vec<Rule>, prop: &str) -> Vec<Rule> {
    rules
        .iter()
        .filter(|r| r.conclusion == prop)
        .cloned()
        .collect()
}

pub fn dualize(rules: &Vec<Rule>) -> Vec<Rule> {
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

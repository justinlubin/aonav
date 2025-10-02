use crate::ao;

use indexmap::IndexSet;

#[derive(Debug, Clone)]
pub struct Rule {
    pub premises: Vec<String>,
    pub conclusion: String,
    pub name: String,
}

impl Rule {
    pub fn axiom(prop: &str) -> Self {
        Self {
            premises: vec![],
            conclusion: prop.to_string(),
            name: format!("ax:{}", prop),
        }
    }
}

pub type ProofSystem = Vec<Rule>;

pub fn to_ao<A, O>(
    ps: ProofSystem,
    target: ao::NodeId,
) -> crate::ao::Graph<A, O> {
    let mut props = IndexSet::new();
    let mut rules = IndexSet::new();
    let mut edges = vec![];
    for rule in ps {
        props.insert(rule.conclusion.clone());
        rules.insert(rule.name.clone());
        edges.push((rule.conclusion.clone(), rule.name.clone()));

        for premise in rule.premises {
            props.insert(premise.clone());
            edges.push((rule.name.clone(), premise.clone()));
        }
    }
    ao::Graph::new(
        props
            .into_iter()
            .map(|id| ao::Node::Or {
                id,
                label: None,
                data: None,
            })
            .chain(rules.into_iter().map(|id| ao::Node::And {
                id,
                label: None,
                data: None,
            })),
        edges.into_iter(),
        &target,
    )
    .unwrap()
}

////////////////////////////////////////////////////////////////////////////////
// Parsing

#[derive(Debug, Clone)]
struct Context<'a> {
    lines: &'a Vec<String>,
    offset: usize,
    rules: Vec<Rule>,
}

impl<'a> Context<'a> {
    fn new(lines: &'a Vec<String>) -> Self {
        Context {
            lines,
            offset: 0,
            rules: vec![],
        }
    }

    fn next(&mut self) -> &'a str {
        let line = &self.lines[self.offset];
        self.offset += 1;
        return line;
    }

    fn proof_system(&mut self) {
        while self.offset < self.lines.len() {
            self.command()
        }
    }

    fn command(&mut self) {
        let line = self.next();
        if line.starts_with("axiom:") {
            self.axiom();
        } else if line.starts_with("rule:") {
            self.idb();
        }
    }

    fn axiom(&mut self) {
        let line = self.next();
        self.rules.extend(line.split_whitespace().map(Rule::axiom))
    }

    fn idb(&mut self) {
        let premises_line = self.next();
        let name_line = self.next();
        let conclusion_line = self.next();
        self.rules.push(Rule {
            premises: premises_line
                .split_whitespace()
                .map(String::from)
                .collect(),
            conclusion: conclusion_line.trim().to_string(),
            name: name_line.trim().trim_start_matches("-").trim().to_string(),
        });
    }
}

pub fn proof_system(lines: &Vec<String>) -> ProofSystem {
    let mut p = Context::new(lines);
    p.proof_system();
    p.rules
}

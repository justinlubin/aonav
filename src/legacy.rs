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
    target: ao::NodeLabel,
) -> crate::ao::Graph<A, O> {
    let mut ret = ao::Graph::new(target);
    let mut props = IndexSet::new();
    for rule in ps {
        let aid = ret.add_and_node(rule.name, None);

        let conclusion_oid = if props.insert(rule.conclusion.clone()) {
            ret.add_or_node(rule.conclusion, None)
        } else {
            ret.find_oid(&rule.conclusion)
        };

        ret.add_or_edge(aid, conclusion_oid);

        for premise in rule.premises {
            let oid = if props.insert(premise.clone()) {
                ret.add_or_node(premise, None)
            } else {
                ret.find_oid(&premise)
            };

            ret.add_and_edge(oid, aid);
        }
    }
    ret
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

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

#[derive(Debug, Clone)]
pub enum PSGNode {
    Top,
    Prop(String),
    Rule(String),
}

impl std::fmt::Display for PSGNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Top => "⊤",
            Self::Prop(x) => x,
            Self::Rule(x) => x,
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum PSGEdge {
    And,
    Or,
}

pub type ProofSystemGraph = petgraph::graph::Graph<PSGNode, PSGEdge>;

impl std::fmt::Display for PSGEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::And => "and",
            Self::Or => "or",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct Proof {
    pub premises: Vec<Proof>,
    pub conclusion: String,
    pub rule_name: String,
}

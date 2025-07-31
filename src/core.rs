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
            name: format!("axiom.{}", prop),
        }
    }
}

pub type ProofSystem = Vec<Rule>;

#[derive(Debug, Clone)]
pub enum ProofSystemGraphNode {
    Prop(String),
    Rule(String),
}

pub type ProofSystemGraph = petgraph::graph::Graph<ProofSystemGraphNode, String>;

#[derive(Debug, Clone)]
pub struct Proof {
    pub premises: Vec<Proof>,
    pub conclusion: String,
    pub rule_name: String,
}

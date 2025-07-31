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

#[derive(Debug, Clone)]
pub enum PSGNode {
    Prop(String),
    Rule(String),
}

impl std::fmt::Display for PSGNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
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

use crate::ao::*;
use crate::partition_navigation::*;

pub fn random(graph: &Graph) -> Exp {
    Exp::new(graph.clone())
}

pub fn minimize(e: Exp) -> Exp {
    e.clone()
}

// pub fn random_minimal() -> Exp {
//    minimize(random())
// }

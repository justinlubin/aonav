use crate::ao;
use crate::ao_navigation;

#[derive(Debug)]
pub struct BenchmarkEntry {
    pub name: String,
    pub graph: ao::Graph<(), ()>,
    pub chosen_solutions: Option<Vec<ao_navigation::AxiomSet>>,
}

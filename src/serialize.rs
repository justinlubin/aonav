// serialize egraph to our and/or format

// what are root e-classes and do they matter here

use egg::*;
use indexmap::IndexMap;
use std::{collections::HashSet, fmt::Display, fs::File, io::prelude::*};
use crate::jgf::{self};
use serde_json;

pub fn get_simple_egraph(eg: &mut EGraph<egg::SymbolLang, ()>) {
    //let mut eg: EGraph<SymbolLang, ()> = Default::default();
    let a_class = eg.add(SymbolLang::leaf("a"));
    let b_class = eg.add(SymbolLang::leaf("b"));
    //let ab_class = eg.union(a_class, b_class);
    eg.add(SymbolLang::new("c", vec![a_class, b_class]));
    let ab_class = eg.union(a_class, b_class);
    eg.rebuild();
}


// copy-paste from egraph-serialize
pub fn egg_to_serialized_egraph<L, A>(egraph: &EGraph<L, A>) -> egraph_serialize::EGraph
where
    L: Language + Display,
    A: Analysis<L>,
{
    use egraph_serialize::*;
    let mut out = EGraph::default();
    for class in egraph.classes() {
        for (i, node) in class.nodes.iter().enumerate() {
            out.add_node(
                format!("{}.{}", class.id, i),
                Node {
                    op: node.to_string(),
                    children: node
                        .children()
                        .iter()
                        .map(|id| NodeId::from(format!("{}.0", id)))
                        .collect(),
                    eclass: ClassId::from(format!("{}", class.id)),
                    cost: Cost::new(1.0).unwrap(),
                    subsumed: false,
                },
            )
        }
    }
    out
}


// strongly inspired by egraph-serialize
// serializes egraph into and/or format in ao-examples/name.json
pub fn egg_to_and_or<L, A>(egraph: &EGraph<L, A>, name: String)
where
    L: Language + Display,
    A: Analysis<L>,
{

    let mut edges = Vec::new();
    let mut nodes: IndexMap<String, jgf::Node> = IndexMap::new();
    for class in egraph.classes() {
        // add AND node for class
        let mut and_metadata = IndexMap::new();
        and_metadata.insert(String::from("kind"), serde_json::Value::String(String::from("OR")));
        nodes.insert(class.id.to_string(), jgf::Node {
            label: Some(class.id.to_string()), 
            metadata: Some(and_metadata),
            });
        for (i, node) in class.nodes.iter().enumerate() {
            // add OR node for node
            let mut or_metadata = IndexMap::new();
            or_metadata.insert(String::from("kind"), serde_json::Value::String(String::from("AND")));
            nodes.insert(node.to_string(), jgf::Node {
                label: Some(node.to_string()), 
                metadata: Some(or_metadata),
            });
            // add edge from node to class
            edges.push(jgf::Edge{
                    id: None,
                    source: node.to_string(), 
                    target: class.id.to_string(),
                    relation: None,
                    directed: true,
                    label: None,
                    metadata: None
                    }
                );
            // add edge from each child class to node and avoid duplicate edges
            let mut seen_classes: HashSet<&Id> = HashSet::new();
            for child in node.children() {
                if !seen_classes.contains(child) {
                    seen_classes.insert(child);
                    edges.push(jgf::Edge{
                        id: None,
                        source: child.to_string(),
                        target: node.to_string(),
                        relation: None,
                        directed: true,
                        label: None,
                        metadata: None
                    }
                );
                }
            }
        }
    }

    let and_or_g = jgf::Graph{
        id: Some(String::from(name)),
        label: Some(String::from ("label?")),
        directed: true,
        graph_type: None,
        metadata: None,
        nodes: Some(nodes),
        edges: Some(edges)
    };
    let and_or = jgf::Data::Single { graph: and_or_g };
    let to_json = serde_json::to_string_pretty(&and_or).expect("Failed to go from struct to pretty json");
    let mut file = File::create("ao-examples/from_egg.json").expect("Failed to create file");
    file.write_all(to_json.as_bytes()).expect("Failed to write");
}

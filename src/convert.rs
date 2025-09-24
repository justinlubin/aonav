// serialize egraph to our and/or format

// what are root e-classes and do they matter here

use crate::ao;
use crate::jgf;
use crate::jgf::Edge;
use crate::jgf::Node;

use egg::*;
use indexmap::IndexMap;
use log::info;
use rayon::string;
use serde::Deserialize;
use serde_json;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::hash::RandomState;
use std::os::unix::process::parent_id;
use std::{fmt::Display, fs::File, io::prelude::*};

// create new Graph from args and write to .json
pub fn new_ao(
    id_arg: Option<String>,
    label_arg: Option<String>,
    directed_arg: bool,
    graph_type_arg: Option<String>,
    metadata_arg: Option<IndexMap<String, serde_json::Value>>,
    nodes_arg: Option<IndexMap<String, Node>>,
    edges_arg: Option<Vec<Edge>>,
    file_name: &str,
) {
    let and_or_g = jgf::Graph {
        id: id_arg,
        label: label_arg,
        directed: directed_arg,
        graph_type: graph_type_arg,
        metadata: metadata_arg,
        nodes: nodes_arg,
        edges: edges_arg,
    };
    let and_or = jgf::Data::Single { graph: and_or_g };
    let to_json = serde_json::to_string_pretty(&and_or)
        .expect("Failed to go from struct to pretty json");
    let mut file = File::create(file_name).expect("Failed to create file");
    file.write_all(to_json.as_bytes()).expect("Failed to write");
}

#[allow(dead_code)]
pub fn get_simple_egraph(eg: &mut EGraph<egg::SymbolLang, ()>) {
    //let mut eg: EGraph<SymbolLang, ()> = Default::default();
    let a_class = eg.add(SymbolLang::leaf("a"));
    let b_class = eg.add(SymbolLang::leaf("b"));
    //let ab_class = eg.union(a_class, b_class);
    eg.add(SymbolLang::new("c", vec![a_class, b_class]));
    let _ab_class = eg.union(a_class, b_class);
    eg.rebuild();
}

// copy-paste from egraph-serialize
#[allow(dead_code)]
pub fn egraph_to_serialized_egraph<L, A>(
    egraph: &EGraph<L, A>,
) -> egraph_serialize::EGraph
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

fn insert_node(
    mut nodes: IndexMap<String, jgf::Node>,
    kind: String,
    id: String,
    label: String,
) {
    let mut metadata: IndexMap<String, Value> = IndexMap::new();
    metadata.insert(String::from("kind"), serde_json::Value::String(kind));
    nodes.insert(
        id,
        jgf::Node {
            label: Some(label),
            metadata: Some(metadata),
        },
    );
}

// strongly inspired by egraph-serialize
// serializes egraph into and/or format in ao-examples/name.json
#[allow(dead_code)]
pub fn egraph_to_and_or<L, A>(egraph: &EGraph<L, A>, name: String)
where
    L: Language + Display,
    A: Analysis<L>,
{
    let mut edges = Vec::new();
    let mut nodes: IndexMap<String, jgf::Node> = IndexMap::new();
    for class in egraph.classes() {
        // add OR node for class
        let mut or_metadata = IndexMap::new();
        or_metadata.insert(
            String::from("kind"),
            serde_json::Value::String(String::from("OR")),
        );
        nodes.insert(
            class.id.to_string(),
            jgf::Node {
                label: Some(class.id.to_string()),
                metadata: Some(or_metadata),
            },
        );
        for (_i, node) in class.nodes.iter().enumerate() {
            // add AND node for node
            let mut and_metadata = IndexMap::new();
            and_metadata.insert(
                String::from("kind"),
                serde_json::Value::String(String::from("AND")),
            );
            nodes.insert(
                node.to_string(),
                jgf::Node {
                    label: Some(node.to_string()),
                    metadata: Some(and_metadata),
                },
            );
            // add edge from node to class
            edges.push(jgf::Edge {
                id: None,
                source: node.to_string(),
                target: class.id.to_string(),
                relation: None,
                directed: true,
                label: None,
                metadata: None,
            });
            // add edge from each child class to node and avoid duplicate edges
            let mut seen_classes: HashSet<&Id> = HashSet::new();
            for child in node.children() {
                if !seen_classes.contains(child) {
                    seen_classes.insert(child);
                    edges.push(jgf::Edge {
                        id: None,
                        source: child.to_string(),
                        target: node.to_string(),
                        relation: None,
                        directed: true,
                        label: None,
                        metadata: None,
                    });
                }
            }
        }
    }

    new_ao(
        Some(String::from(name)),
        Some(String::from("label?")),
        true,
        None,
        None,
        Some(nodes),
        Some(edges),
        "ao-examples/from_egg.json",
    );
}

pub fn es_egraph_to_ao(
    _es_egraph: &egraph_serialize::EGraph,
) -> ao::Graph<String, String> {
    todo!()
}

#[derive(serde::Serialize, Deserialize, Debug)]
struct ArgusData {
    root: String,
    parent_to_children: IndexMap<String, Vec<String>>,
    goals: HashSet<String>,
}

pub fn argus_to_and_or(path: &str) {
    let json_data =
        fs::read_to_string("/Users/marlenapreigh/under/examples/argus.json")
            .expect("Failed to read file");
    let deserialized: ArgusData =
        serde_json::from_str(&json_data).expect("Failed to deserialize JSON");

    let mut edges = Vec::new();
    let mut nodes: IndexMap<String, jgf::Node> = IndexMap::new();
    // for each parent, add edge to each child
    for (parent, children) in deserialized.parent_to_children.iter() {
        let parent_id = parent.to_string();
        let parent_id_copy = parent_id.clone();
        // if node is key in goal_info it's an OR node and the others are AND nodes
        if !nodes.contains_key(&parent_id) {
            if deserialized.goals.contains(&parent_id) {
                // OR ndoe
                let mut metadata: IndexMap<String, Value> = IndexMap::new();
                metadata.insert(
                    String::from("kind"),
                    serde_json::Value::String("OR".to_string()),
                );
                nodes.insert(
                    parent_id.clone(),
                    jgf::Node {
                        label: Some(parent_id.clone()),
                        metadata: Some(metadata),
                    },
                );
            } else {
                // AND node
                let mut metadata: IndexMap<String, Value> = IndexMap::new();
                metadata.insert(
                    String::from("kind"),
                    serde_json::Value::String("AND".to_string()),
                );
                nodes.insert(
                    parent_id.clone(),
                    jgf::Node {
                        label: Some(parent_id.clone()),
                        metadata: Some(metadata),
                    },
                );
            }
        }
        for child in children {
            let mut child_label: String = "".to_string();
            let child_id = child.to_string();
            if !nodes.contains_key(&child_id) {
                if deserialized.goals.contains(&child_id) {
                    // OR ndoe
                    let mut metadata: IndexMap<String, Value> = IndexMap::new();
                    metadata.insert(
                        String::from("kind"),
                        serde_json::Value::String("OR".to_string()),
                    );
                    nodes.insert(
                        child_id.clone(),
                        jgf::Node {
                            label: Some(child_id.clone()),
                            metadata: Some(metadata),
                        },
                    );
                } else {
                    // AND node
                    let mut metadata: IndexMap<String, Value> = IndexMap::new();
                    metadata.insert(
                        String::from("kind"),
                        serde_json::Value::String("AND".to_string()),
                    );
                    nodes.insert(
                        child_id.clone(),
                        jgf::Node {
                            label: Some(child_id.clone()),
                            metadata: Some(metadata),
                        },
                    );
                }
            }
            edges.push(jgf::Edge {
                id: None,
                source: parent_id.clone(),
                target: child_id.clone(),
                relation: None,
                directed: true,
                label: None,
                metadata: None,
            });
        }
    }

    let mut md: IndexMap<String, Value> = IndexMap::new();
    md.insert(
        "goal".to_string(),
        serde_json::Value::String(deserialized.root),
    );

    new_ao(
        Some("id".to_string()),
        Some("label".to_string()),
        true,
        Some("type".to_string()),
        Some(md),
        Some(nodes),
        Some(edges),
        "ao-examples/from_argus.json",
    );

    // double check ArgusData structure has right stuff
    /*let to_json = serde_json::to_string_pretty(&deserialized).expect("failed");
    let mut file = File::create("ao-examples/from_argus.json")
        .expect("Failed to create file");
    file.write_all(to_json.as_bytes()).expect("Failed to write");*/
}

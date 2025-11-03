use crate::ao::*;

use crate::jgf;
use crate::util;

use egg::*;
use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////
// JSON Graph Format

impl TryFrom<jgf::Graph> for Graph {
    type Error = String;

    fn try_from(value: jgf::Graph) -> Result<Self, Self::Error> {
        let jgf_nodes = value.nodes.ok_or("Missing graph nodes")?;
        let jgf_edges = value.edges.ok_or("Missing graph edges")?;

        let goal = value
            .metadata
            .ok_or("Missing graph metadata")?
            .get("goal")
            .ok_or("Missing 'goal' metadata for graph")?
            .as_str()
            .ok_or("'goal' metadata for graph is not a string")?
            .to_owned();

        let mut nodes = vec![];

        for (node_id, node_val) in jgf_nodes {
            let metadata = node_val
                .metadata
                .ok_or(format!("Missing metadata for node '{}'", node_id))?;
            let kind = match metadata
                .get("kind")
                .ok_or(format!(
                    "Missing 'kind' metadata for node '{}'",
                    node_id
                ))?
                .as_str()
                .ok_or(format!(
                    "'kind' metadata for node '{}' is not a string",
                    node_id
                ))?
                .to_ascii_uppercase()
                .as_str()
            {
                "AND" => NodeKind::And,
                "OR" => NodeKind::Or,
                k => {
                    return Err(format!(
                        "Unknown 'kind' metadata '{}' for node '{}'",
                        k, node_id
                    ))
                }
            };
            nodes.push(Node::new(node_id, node_val.label, kind));
        }

        Ok(Graph::new(
            nodes.into_iter(),
            jgf_edges.into_iter().map(|e| (e.source, e.target)),
            &goal,
        )?)
    }
}

impl TryFrom<Graph> for jgf::Graph {
    type Error = String;

    fn try_from(ao: Graph) -> Result<Self, Self::Error> {
        let mut nodes = IndexMap::new();

        for node in ao.nodes() {
            nodes.insert(
                node.id().to_owned(),
                jgf::Node {
                    label: node.label().map(|x| x.to_owned()),
                    metadata: Some(IndexMap::from([(
                        "kind".to_owned(),
                        serde_json::Value::String(node.kind().to_string()),
                    )])),
                },
            );
        }

        Ok(jgf::Graph {
            id: None,
            label: None,
            directed: true,
            graph_type: None,
            metadata: Some(IndexMap::from([(
                "goal".to_owned(),
                serde_json::Value::String(ao.or_at(ao.goal()).id().to_owned()),
            )])),
            nodes: Some(nodes),
            edges: Some(
                ao.edges()
                    .map(|(source, target)| jgf::Edge {
                        id: None,
                        source: source.id().to_owned(),
                        target: target.id().to_owned(),
                        relation: None,
                        directed: true,
                        label: None,
                        metadata: None,
                    })
                    .collect(),
            ),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// egglog

fn egglog_or_id(relation: &str, arguments: &Vec<i64>) -> String {
    format!(
        "{}({})",
        relation,
        arguments
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn egglog_and_id(
    head: &str,
    vars: &Vec<String>,
    substitutions: &IndexMap<String, i64>,
) -> String {
    format!(
        "{} @ {}",
        head,
        vars.iter()
            .map(|x| format!("{}={}", x, substitutions[x]))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn sorted_rule_vars(
    head: &egglog::ast::GenericExpr<String, String>,
    body: &Vec<egglog::ast::GenericFact<String, String>>,
) -> Vec<String> {
    let mut set: HashSet<_> = head.vars().collect();
    for f in body {
        match f {
            egglog::ast::GenericFact::Eq(_, e1, e2) => {
                set.extend(e1.vars().chain(e2.vars()))
            }
            egglog::ast::GenericFact::Fact(e) => set.extend(e.vars()),
        };
    }
    let mut vec: Vec<_> = set.into_iter().collect();
    vec.sort();
    vec
}

// fn head_var(
//     e: &egglog::ast::GenericExpr<String, String>,
// ) -> Result<String, String> {
//     match e {
//         egglog::ast::GenericExpr::Call(_, head, _) => Ok(head.clone()),
//         _ => Err(format!("Unsupported head expression '{}'", e)),
//     }
// }

fn ground_args(
    args: &Vec<egglog::ast::GenericExpr<String, String>>,
) -> Result<Vec<i64>, String> {
    let mut ret = vec![];
    for arg in args {
        match arg {
            egglog::ast::GenericExpr::Lit(_, egglog::ast::Literal::Int(x)) => {
                ret.push(*x)
            }
            _ => {
                return Err(format!(
                    "{} not a supported ground literal in {:?}",
                    arg, args
                ))
            }
        }
    }
    Ok(ret)
}

fn ground_fact(
    e: &egglog::ast::GenericExpr<String, String>,
) -> Result<(String, Vec<i64>), String> {
    match e {
        egglog::ast::GenericExpr::Call(_, head, args) => {
            Ok((head.clone(), ground_args(args)?))
        }
        _ => Err(format!("'{}' not a ground fact", e)),
    }
}

impl TryFrom<Vec<egglog::ast::Command>> for Graph {
    type Error = String;

    fn try_from(
        egglog_program: Vec<egglog::ast::Command>,
    ) -> Result<Self, Self::Error> {
        // Extract relevant parts of egglog program: relations, rules, checks

        let mut relations = vec![];
        let mut rules = vec![];
        let mut checks = vec![];

        for (i, cmd) in egglog_program.into_iter().enumerate() {
            match cmd {
                egglog::ast::Command::Relation { name, inputs, .. } => {
                    relations.push((name, inputs))
                }
                egglog::ast::Command::Rule { mut rule } => {
                    if rule.head.0.len() != 1 {
                        return Err(format!(
                            "Head size must be 1 for '{}'",
                            rule
                        ));
                    }

                    let head = match rule.head.0.swap_remove(0) {
                        egglog::ast::GenericAction::Expr(_, e) => e,
                        h => return Err(format!("Unsupported head '{}'", h)),
                    };

                    let name = if rule.name.is_empty() {
                        format!("rule{}", i)
                    } else {
                        rule.name
                    };

                    rules.push((name, head, rule.body));
                }
                egglog::ast::Command::Check(_, check) => checks.push(check),
                egglog::ast::GenericCommand::RunSchedule(_) => (),
                _ => return Err(format!("Unsupported command '{}'", cmd)),
            };
        }

        // Make sure there's exactly 1 check, and of the right form

        if checks.len() != 1 {
            return Err(format!(
                "Must have exactly 1 check, not {}",
                checks.len()
            ));
        }

        let mut supercheck = checks.swap_remove(0);

        if supercheck.len() != 1 {
            return Err(format!(
                "Must have exactly 1 check in check, not {}",
                checks.len()
            ));
        }

        let check = supercheck.swap_remove(0);

        let (check_relation, check_arguments) =
            match check {
                egglog::ast::GenericFact::Fact(
                    egglog::ast::GenericExpr::Call(_, head, body),
                ) => {
                    let mut args = vec![];
                    for e in body {
                        match e {
                            egglog::ast::GenericExpr::Lit(
                                _,
                                egglog::ast::Literal::Int(x),
                            ) => args.push(x),
                            _ => {
                                return Err(format!(
                                "Unsupported body expression in check: '{}'",
                                e
                            ))
                            }
                        }
                    }
                    (head, args)
                }
                _ => return Err(format!("Unsupported check type")),
            };

        // Calculate domain

        let mut domain: IndexSet<_> = check_arguments.iter().cloned().collect();
        let mut supported_domain = true;

        for (_, head, body) in &rules {
            let mut roots = vec![head];
            for fact in body {
                match fact {
                    egglog::ast::GenericFact::Eq(_, e1, e2) => {
                        roots.extend(vec![e1, e2])
                    }
                    egglog::ast::GenericFact::Fact(e) => roots.push(e),
                };
            }
            for root in roots {
                root.walk(
                    &mut |e| {
                        match e {
                            egglog::ast::GenericExpr::Lit(_, lit) => {
                                match lit {
                                    egglog::ast::Literal::Int(x) => {
                                        domain.insert(*x);
                                    }
                                    _ => supported_domain = false,
                                }
                            }
                            _ => (),
                        };
                    },
                    &mut |_| {},
                )
            }
        }

        if !supported_domain {
            return Err(format!("Unsupported types for domain"));
        }

        // Compute nodes

        let mut nodes = vec![];

        // OR nodes: Ground all relations and find goal

        let mut goal = None;

        for (relation, params) in relations {
            let mut choices = IndexMap::new();
            for (i, param) in params.into_iter().enumerate() {
                if param != "i64" {
                    return Err(format!(
                        "Unsupported parameter type for relation '{}': '{}'",
                        relation, param
                    ));
                }
                let _ = choices.insert(i, domain.iter().cloned().collect());
            }
            for arguments in
                util::cartesian_product(&util::Timer::infinite(), choices)
                    .unwrap()
            {
                let arguments: Vec<_> = arguments.into_values().collect();
                let id = egglog_or_id(&relation, &arguments);
                if relation == check_relation && arguments == check_arguments {
                    goal = Some(id.clone());
                }
                nodes.push(Node::new(id, None, NodeKind::Or));
            }
        }

        let goal = goal.ok_or_else(|| "Could not find goal")?;

        // AND nodes: Ground all rules (also create edges)

        let mut edges = vec![];

        for (name, head, body) in rules {
            let mut choices = IndexMap::new();
            let vars = sorted_rule_vars(&head, &body);
            for var in &vars {
                let _ = choices
                    .insert(var.clone(), domain.iter().cloned().collect());
            }
            for substitutions in
                util::cartesian_product(&util::Timer::infinite(), choices)
                    .unwrap()
            {
                let id = egglog_and_id(&name, &vars, &substitutions);

                let mut lookup = |s: &egglog::ast::Span,
                                  x: &String|
                 -> egglog::ast::GenericExpr<
                    String,
                    String,
                > {
                    egglog::ast::GenericExpr::Lit(
                        s.clone(),
                        egglog::ast::Literal::Int(substitutions[x]),
                    )
                };

                for f in &body {
                    match f {
                        egglog::ast::GenericFact::Fact(e) => {
                            let (relation, arguments) =
                                ground_fact(&e.subst_leaf(&mut lookup))?;
                            let premise = egglog_or_id(&relation, &arguments);
                            edges.push((id.clone(), premise));
                        }
                        egglog::ast::GenericFact::Eq(..) => (),
                    }
                }

                let ground_head = head.subst_leaf(&mut lookup);

                let (head_relation, head_arguments) =
                    ground_fact(&ground_head)?;
                let conclusion = egglog_or_id(&head_relation, &head_arguments);
                edges.push((conclusion, id.clone()));

                nodes.push(Node::new(id, None, NodeKind::And));
            }
        }

        // Return graph

        Graph::new(nodes.into_iter(), edges.into_iter(), &goal)
    }
}

////////////////////////////////////////////////////////////////////////////////

// serialize egraph to our and/or format

// what are root e-classes and do they matter here

// create new Graph from args and write to .json
pub fn new_ao(
    id_arg: Option<String>,
    label_arg: Option<String>,
    directed_arg: bool,
    graph_type_arg: Option<String>,
    metadata_arg: Option<IndexMap<String, serde_json::Value>>,
    nodes_arg: Option<IndexMap<String, jgf::Node>>,
    edges_arg: Option<Vec<jgf::Edge>>,
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

#[allow(dead_code)]
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

pub fn es_egraph_to_ao(_es_egraph: &egraph_serialize::EGraph) -> Graph {
    todo!()
}

#[derive(serde::Serialize, Deserialize, Debug)]
struct ArgusAO {
    root: String,
    goals: HashMap<String, String>,
    candidates: HashMap<String, String>,
    topology: HashMap<String, Vec<String>>,
    yesGoals: Vec<String>
}

pub fn argus_to_and_or<A, O>(path: &PathBuf) 
    where jgf::Graph: TryFrom<Graph> {
    let json_data = std::fs::read_to_string(path).expect("Failed to read file");
    let deserialized: ArgusAO =
        serde_json::from_str(&json_data).expect("Failed to deserialize JSON");

        // go through edges, creating nodes for unseen ids, and creating edges
        let mut nodes: Vec<Node> = Vec::new();
        let mut edges: Vec<(NodeId, NodeId)> = Vec::new();
        let goal = deserialized.root;

        // if both are goals, remove child from graph
        // or if parent has been removed, remove child as well- doesn't work because ids aren't visited perfectly in order
        // 
        let mut removed: HashSet<String> = HashSet::new();
        let mut edges_removed: HashSet<(String, String)> = HashSet::new();
        let mut new_rule_id = -1;
        for (parent, children) in &deserialized.topology {
            for child in children {
                if deserialized.goals.contains_key(parent) && deserialized.goals.contains_key(child) && !removed.contains(child){
                    //print!("\nremoving goal {}: {:?}\n parent was goal {}: {:?}\n", child, deserialized.goals.get(child), parent, deserialized.goals.get(parent));
                    print!("\n adding rule between parent {:?} and child {:?}\n", parent, child);
                    nodes.push(Node::new(new_rule_id.to_string(), None, NodeKind::And));
                    edges.push((parent.to_string(), new_rule_id.to_string()));
                    edges.push((new_rule_id.to_string(), child.to_string()));
                    new_rule_id -= 1;
                    edges_removed.insert((parent.to_string(), child.to_string()));
                    //removed.insert(child.to_string());
                }
            }
        }

        // goal nodes
        for (goal_id, goal_label) in &deserialized.goals {
            if !removed.contains(goal_id) {
                nodes.push(Node::new(goal_id.to_string(), Some(goal_label.to_string()), NodeKind::Or));
            }
        }
        // candidate nodes
        for (candidate_id, candidate_label) in deserialized.candidates {
            nodes.push(Node::new(candidate_id.to_string(), Some(candidate_label.to_string()), NodeKind::And));
        }

        // edges
        for (parent, children) in deserialized.topology {
            for child in children {
                if !removed.contains(&child) && !edges_removed.contains(&(parent.clone(), child.clone())) {
                    edges.push((parent.clone(), child));
                }
            }
        }
        // for each goals this is true, add empty impl going into it
        for true_goal in deserialized.yesGoals {
            if !removed.contains(&true_goal) {
                //nodes.push(Node::new("-".to_owned() + &true_goal, None, NodeKind::And));
                nodes.push(Node::new(new_rule_id.to_string(), None, NodeKind::And));
                edges.push((true_goal.clone(), new_rule_id.to_string()));
                new_rule_id -= 1;
                //edges.push((true_goal.clone(), "-".to_owned() + &true_goal));
            }
        }
        // get ao graph
        let graph = Graph::new(nodes.into_iter(), edges.into_iter(), &goal).unwrap();
        // go to jgf and write to file
        let jgf = jgf::Graph::try_from(graph);
        match jgf {
            Ok(g) => {
                let to_json = serde_json::to_string_pretty(&g).expect("Failed to go from struct to pretty json");
                let mut file = File::create("argus-examples/argus-ao-read.json").expect("Failed to create file");
                file.write_all("{ \"graph\": ".as_bytes()).expect("Failed to write");
                file.write_all(to_json.as_bytes()).expect("Failed to write");
                file.write_all("}".as_bytes()).expect("Failed to write");
            },
            Err(e) => {panic!("Failed");}
        }
        
}

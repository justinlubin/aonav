use colored::Colorize;
use indexmap::{IndexMap, IndexSet};
use std::io::Write;

// Utilities

pub fn cartesian_product<K: Clone + Eq + std::hash::Hash, V: Clone>(
    choices: IndexMap<K, Vec<V>>,
) -> Vec<IndexMap<K, V>> {
    let mut results = vec![IndexMap::new()];
    for (k, vs) in choices.iter() {
        let mut new_results = vec![];
        for map in results {
            for v in vs {
                let mut new_map = map.clone();
                new_map.insert(k.clone(), v.clone());
                new_results.push(new_map)
            }
        }
        results = new_results;
    }
    results
}

fn load_lines(path: &str) -> Option<Vec<String>> {
    match std::fs::read_to_string(&path) {
        Ok(s) => Some(s.lines().map(String::from).collect()),
        Err(_) => None,
    }
}

// Proof systems (programs)

#[derive(Debug, Clone)]
struct Rule {
    premises: Vec<String>,
    conclusion: String,
    name: String,
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

// Parsing

#[derive(Debug, Clone)]
struct Parser<'a> {
    lines: &'a Vec<String>,
    offset: usize,
    rules: Vec<Rule>,
}

impl<'a> Parser<'a> {
    pub fn new(lines: &'a Vec<String>) -> Self {
        Parser {
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

    fn program(&mut self) {
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
            premises: premises_line.split_whitespace().map(String::from).collect(),
            conclusion: conclusion_line.trim().to_string(),
            name: name_line.trim().trim_start_matches("-").trim().to_string(),
        });
    }
}

fn parse(lines: &Vec<String>) -> Vec<Rule> {
    let mut p = Parser::new(lines);
    p.program();
    p.rules
}

// Unparsing / pretty-printing

fn unparse_rule(rule: &Rule) -> String {
    let premises = rule.premises.join("   ");
    let conclusion = &rule.conclusion;
    let dash_count = std::cmp::max(premises.len(), conclusion.len()) + 2;
    format!(
        "  {: ^width$}\n  {} {}\n  {: ^width$}",
        premises.magenta(),
        "─".repeat(dash_count),
        rule.name.yellow(),
        conclusion.green(),
        width = dash_count,
    )
}

fn unparse(rules: &Vec<Rule>) -> String {
    rules
        .iter()
        .map(|r| format!("{}\n{}", "rule:".bright_black(), unparse_rule(r)))
        .collect::<Vec<_>>()
        .join("\n\n")
}

// Modifications to proof systems

fn get_props(rules: &Vec<Rule>) -> IndexSet<String> {
    rules
        .iter()
        .flat_map(|r| std::iter::once(r.conclusion.clone()).chain(r.premises.iter().cloned()))
        .collect()
}

fn get(rules: &Vec<Rule>, prop: &str) -> Vec<Rule> {
    rules
        .iter()
        .filter(|r| r.conclusion == prop)
        .cloned()
        .collect()
}

fn dualize(rules: &Vec<Rule>) -> Vec<Rule> {
    rules
        .iter()
        .flat_map(|r| {
            r.premises.iter().enumerate().map(|(i, p)| Rule {
                premises: vec![format!("un {}", p)],
                conclusion: format!("na {}", r.name),
                name: format!("na.{}.{}", r.name, i + 1),
            })
        })
        .chain(get_props(rules).into_iter().map(|p| {
            Rule {
                premises: get(rules, &p)
                    .into_iter()
                    .map(|r| format!("na {}", r.name))
                    .collect(),
                conclusion: format!("un {}", p),
                name: format!("un.{}", p),
            }
        }))
        .collect()
}

// Proofs and proof search

#[derive(Debug, Clone)]
struct Proof {
    premises: Vec<Proof>,
    conclusion: String,
    rule_name: String,
}

impl Proof {
    pub fn print(&self, indent: usize) {
        println!(
            "{} (by {})",
            self.conclusion.green(),
            self.rule_name.yellow()
        );
        for prem in &self.premises {
            print!("{} • ", "  ".repeat(indent));
            prem.print(indent + 1);
        }
    }
}

fn prove(rules: &Vec<Rule>, prop: &str) -> Vec<Proof> {
    get(rules, prop)
        .into_iter()
        .flat_map(|r| {
            cartesian_product(
                r.premises
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| (i, prove(rules, &p)))
                    .collect(),
            )
            .into_iter()
            .map(move |subproofs| Proof {
                premises: subproofs.into_iter().map(|(_, x)| x).collect(),
                conclusion: prop.to_string(),
                rule_name: r.name.clone(),
            })
        })
        .collect()
}

// Interactive session

#[derive(Debug, Clone)]
enum Command {
    Quit,
    Load { path: String },
    LoadExample { example: String },
    Prove { prop: String },
    Dualize,
    RemoveRule { rule: String },
    RemovePremise { rule: String, premise: usize },
    AddAxiom { prop: String },
    DisplayCommand,
    Help,
    Nop,
}

fn next_arg(it: &mut std::str::Split<&str>) -> Option<String> {
    Some(it.next()?.to_string())
}

fn rest_arg(it: std::str::Split<&str>) -> Option<String> {
    let rest = it.collect::<Vec<_>>();
    if rest.is_empty() {
        None
    } else {
        Some(rest.join(" "))
    }
}

fn parse_command(line: &str) -> Result<Command, String> {
    let mut it = line.split(" ");

    let name = next_arg(&mut it).unwrap();

    Ok(if name == "quit" || name == "q" {
        Command::Quit
    } else if name == "load" || name == "l" {
        let path = rest_arg(it).ok_or("syntax: load <path>".to_string())?;
        Command::Load { path }
    } else if name == "loadexample" || name == "lex" {
        let example = rest_arg(it).ok_or("syntax: loadexample <example name>".to_string())?;
        Command::LoadExample { example }
    } else if name == "dualize" || name == "d" {
        Command::Dualize
    } else if name == "prove" || name == "p" {
        let prop = rest_arg(it).ok_or("syntax: prove <proposition name>".to_string())?;
        Command::Prove { prop }
    } else if name == "removerule" || name == "rr" {
        let rule = rest_arg(it).ok_or("syntax: removerule <rule name>".to_string())?;
        Command::RemoveRule { rule }
    } else if name == "removepremise" || name == "rp" {
        let err = "syntax: removepremise <rule name> <premise name>";
        let rule = next_arg(&mut it).ok_or(err.to_string())?;
        let premise = rest_arg(it)
            .ok_or(err.to_string())?
            .parse::<usize>()
            .map_err(|_| err.to_string())?;
        Command::RemovePremise { rule, premise }
    } else if name == "addaxiom" || name == "aa" {
        let prop = rest_arg(it).ok_or("syntax: addaxiom <proposition name>".to_string())?;
        Command::AddAxiom { prop }
    } else if name == "displaycommand" {
        Command::DisplayCommand
    } else if name == "help" || name == "h" {
        Command::Help
    } else {
        Command::Nop
    })
}

struct Session {
    program: Vec<Rule>,
    complete: bool,
    display_command: bool,
}

impl Session {
    fn new() -> Self {
        Session {
            program: vec![],
            complete: false,
            display_command: false,
        }
    }

    fn go(&mut self) {
        while !self.complete {
            print!("> ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            };
            let input = input.trim();

            if self.display_command {
                println!("{}", input);
            }

            let command = match parse_command(&input) {
                Ok(c) => c,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };

            self.exec(command)
        }
    }

    fn show_program(&self) {
        println!("{}", unparse(&self.program));
    }

    fn exec(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => self.complete = true,
            Command::Load { path } => {
                let lines = match load_lines(&path) {
                    Some(lines) => lines,
                    None => {
                        println!("file not found");
                        return;
                    }
                };

                self.program = parse(&lines);
                self.show_program();
            }
            Command::LoadExample { example } => {
                let path = format!("examples/{}.txt", example);

                let lines = match load_lines(&path) {
                    Some(lines) => lines,
                    None => {
                        println!("file not found");
                        return;
                    }
                };

                self.program = parse(&lines);
                self.show_program();
            }
            Command::Prove { prop } => {
                let proofs = prove(&self.program, &prop);

                if proofs.is_empty() {
                    println!("{}", "no proofs".red())
                }

                let mut first = true;
                for (i, p) in proofs.iter().enumerate() {
                    if !first {
                        println!("");
                    }
                    let title = format!("proof {}:", i + 1).bright_black();
                    print!("{}\n  ", title);
                    p.print(1);
                    first = false;
                }
            }
            Command::Dualize => {
                self.program = dualize(&self.program);
                self.show_program();
            }
            Command::RemoveRule { rule } => {
                self.program.retain(|r| r.name != rule);
                self.show_program();
            }
            Command::RemovePremise { rule, premise } => {
                self.program = self
                    .program
                    .clone()
                    .into_iter()
                    .map(|r| {
                        if r.name == rule {
                            Rule {
                                premises: r
                                    .premises
                                    .into_iter()
                                    .enumerate()
                                    .filter_map(
                                        |(i, p)| {
                                            if i + 1 == premise {
                                                None
                                            } else {
                                                Some(p)
                                            }
                                        },
                                    )
                                    .collect(),
                                ..r
                            }
                        } else {
                            r
                        }
                    })
                    .collect();

                self.show_program();
            }
            Command::AddAxiom { prop } => {
                self.program.push(Rule::axiom(&prop));
                self.show_program()
            }
            Command::DisplayCommand => {
                self.display_command = true;
                println!("displaycommand");
            }
            Command::Help => {
                println!("available commands:");

                println!("{}", "loading".green());
                println!("  load (l)");
                println!("  loadexample (lex)");

                println!("\n{}", "proving".green());
                println!("  prove (p)");

                println!("\n{}", "proof system modification".green());
                println!("  dualize (d)");
                println!("  remove (r)");
                println!("  removepremise (rp)");
                println!("  addaxiom (aa)");

                println!("\n{}", "misc.".green());
                println!("  displaycommand");
                println!("  help (h)");
                println!("  quit (q)");
            }
            Command::Nop => (),
        }
    }
}

// Main

fn main() {
    Session::new().go()
}

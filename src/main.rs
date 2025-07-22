use colored::Colorize;
use indexmap::{IndexMap, IndexSet};
use std::io::Write;

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

#[derive(Debug, Clone)]
struct Rule {
    premises: Vec<String>,
    conclusion: String,
    name: String,
}

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
        self.rules.extend(line.split_whitespace().map(|x| Rule {
            premises: vec![],
            conclusion: x.to_string(),
            name: format!("axiom.{}", x),
        }))
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

fn unparse_rule(rule: &Rule) -> String {
    let premises = rule.premises.join("   ");
    let conclusion = &rule.conclusion;
    let dash_count = std::cmp::max(premises.len(), conclusion.len()) + 2;
    format!(
        "  {: ^width$}\n  {} {}\n  {: ^width$}",
        premises.magenta(),
        "-".repeat(dash_count),
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

fn check(command: &Vec<String>, expected_args: usize) -> bool {
    if command.len() - 1 != expected_args {
        let plural = if expected_args == 1 { "" } else { "s" };
        println!("expected {} argument{}", expected_args, plural);
        return false;
    }
    true
}

fn load(path: &str) -> Option<Vec<String>> {
    match std::fs::read_to_string(&path) {
        Ok(s) => Some(s.lines().map(String::from).collect()),
        Err(_) => None,
    }
}

fn main() {
    let mut display_command = false;
    let mut current_prog = vec![];
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
        let input = input.trim();

        if display_command {
            println!("{}", input);
        }

        if input == "q" || input == "quit" {
            break;
        }

        let mut it = input.splitn(2, " ");

        let cmd = match it.next() {
            Some(x) => x,
            None => continue,
        };

        if cmd == "l" || cmd == "load" {
            let rest = match it.next() {
                Some(x) => x,
                None => {
                    println!("syntax: load <path>");
                    continue;
                }
            };

            let lines = match load(rest) {
                Some(lines) => lines,
                None => {
                    println!("file not found");
                    continue;
                }
            };

            current_prog = parse(&lines);
            println!("{}", unparse(&current_prog));
        } else if cmd == "lex" || cmd == "loadexample" {
            let rest = match it.next() {
                Some(x) => x,
                None => {
                    println!("syntax: loadexample <example name>");
                    continue;
                }
            };

            let path = format!("examples/{}.txt", rest);

            let lines = match load(&path) {
                Some(lines) => lines,
                None => {
                    println!("file not found");
                    continue;
                }
            };

            current_prog = parse(&lines);
            println!("{}", unparse(&current_prog));
        } else if cmd == "d" || cmd == "dualize" {
            current_prog = dualize(&current_prog);
            println!("{}", unparse(&current_prog));
        } else if cmd == "p" || cmd == "prove" {
            let rest = match it.next() {
                Some(x) => x,
                None => {
                    println!("syntax: prove <proposition name>");
                    continue;
                }
            };

            let proofs = prove(&current_prog, &rest);

            if proofs.is_empty() {
                println!("{}", "no proofs".red())
            }

            let mut first = true;
            for (i, p) in prove(&current_prog, &rest).iter().enumerate() {
                if !first {
                    println!("");
                }
                let title = format!("proof {}:", i + 1).bright_black();
                print!("{}\n  ", title);
                p.print(1);
                first = false;
            }
        } else if cmd == "r" || cmd == "remove" {
            let rest = match it.next() {
                Some(x) => x,
                None => {
                    println!("syntax: remove <rule name>");
                    continue;
                }
            };

            current_prog = current_prog
                .into_iter()
                .filter(|r| r.name != rest)
                .collect();

            println!("{}", unparse(&current_prog));
        } else if cmd == "displaycommand" {
            display_command = true;
            println!("displaycommand");
        } else if cmd == "h" || cmd == "help" {
            println!("available commands:");

            println!("{}", "loading".green());
            println!("  load (l)");
            println!("  loadexample (lex)");

            println!("\n{}", "proving".green());
            println!("  prove (p)");

            println!("\n{}", "proof system modification".green());
            println!("  dualize (d)");
            println!("  remove (r)");

            println!("\n{}", "misc.".green());
            println!("  displaycommand");
            println!("  help (h)");
            println!("  quit (q)");
        } else if cmd == "" {
            continue;
        } else {
            println!("unrecognized command");
            continue;
        }
    }
}

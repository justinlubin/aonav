use crate::core::*;
use crate::util;
use crate::{parse, prove, transform, unparse};

use colored::Colorize;
use indexmap::IndexSet;

// Line completion

#[derive(rustyline::Hinter, rustyline::Highlighter, rustyline::Validator)]
struct SessionLineHelper {
    props: IndexSet<String>,
}

impl<'a> rustyline::completion::Completer for SessionLineHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &rustyline::Context,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let s = &line[0..pos];
        let start = match s.rfind(" ") {
            None => return Ok((pos, vec![])),
            Some(x) => x + 1,
        };

        if line.starts_with("lex") {
            let mut candidates = vec![];
            for path in std::fs::read_dir("./examples").unwrap() {
                candidates.push(
                    path.unwrap()
                        .path()
                        .with_extension("")
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                );
            }
            return Ok((start, candidates));
        }

        let prefix = &s[start..pos];
        let candidates = self
            .props
            .clone()
            .into_iter()
            .filter(|p| p.starts_with(prefix))
            .collect();
        Ok((start, candidates))
    }
}

impl rustyline::Helper for SessionLineHelper {}

// Commands

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

    if name == "quit" || name == "q" {
        Ok(Command::Quit)
    } else if name == "load" || name == "l" {
        let path = rest_arg(it).ok_or("syntax: load <path>".to_string())?;
        Ok(Command::Load { path })
    } else if name == "loadexample" || name == "lex" {
        let example = rest_arg(it).ok_or("syntax: loadexample <example name>".to_string())?;
        Ok(Command::LoadExample { example })
    } else if name == "dualize" || name == "d" {
        Ok(Command::Dualize)
    } else if name == "prove" || name == "p" {
        let prop = rest_arg(it).ok_or("syntax: prove <proposition name>".to_string())?;
        Ok(Command::Prove { prop })
    } else if name == "removerule" || name == "rr" {
        let rule = rest_arg(it).ok_or("syntax: removerule <rule name>".to_string())?;
        Ok(Command::RemoveRule { rule })
    } else if name == "removepremise" || name == "rp" {
        let err = "syntax: removepremise <rule name> <premise name>";
        let rule = next_arg(&mut it).ok_or(err.to_string())?;
        let premise = rest_arg(it)
            .ok_or(err.to_string())?
            .parse::<usize>()
            .map_err(|_| err.to_string())?;
        Ok(Command::RemovePremise { rule, premise })
    } else if name == "addaxiom" || name == "aa" {
        let prop = rest_arg(it).ok_or("syntax: addaxiom <proposition name>".to_string())?;
        Ok(Command::AddAxiom { prop })
    } else if name == "displaycommand" {
        Ok(Command::DisplayCommand)
    } else if name == "help" || name == "h" {
        Ok(Command::Help)
    } else {
        Err("unrecognized command".to_string())
    }
}

// Sessions

pub struct Session {
    proof_system: ProofSystem,
    complete: bool,
    display_command: bool,
}

impl Session {
    pub fn new() -> Self {
        Session {
            proof_system: vec![],
            complete: false,
            display_command: false,
        }
    }

    pub fn go(&mut self) {
        let mut rl: rustyline::Editor<SessionLineHelper, _> = rustyline::Editor::with_history(
            rustyline::Config::builder().auto_add_history(true).build(),
            rustyline::history::MemHistory::new(),
        )
        .unwrap();

        while !self.complete {
            rl.set_helper(Some(SessionLineHelper {
                props: transform::props(&self.proof_system),
            }));

            let input = match rl.readline("> ") {
                Ok(line) => line.trim().to_string(),
                Err(_) => break,
            };

            if input.is_empty() {
                continue;
            }

            if self.display_command {
                println!("{}", input);
            }

            if input.starts_with("#") {
                continue;
            }

            let command = match parse_command(&input) {
                Ok(c) => c,
                Err(e) => {
                    println!("{}", e.red());
                    continue;
                }
            };

            self.exec(command)
        }
    }

    fn show_proof_system(&self) {
        println!("{}", unparse::proof_system(&self.proof_system));
    }

    fn exec(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => self.complete = true,
            Command::Load { path } => {
                let lines = match util::read_lines(&path) {
                    Some(lines) => lines,
                    None => {
                        println!("file not found");
                        return;
                    }
                };

                self.proof_system = parse::proof_system(&lines);
                self.show_proof_system();
            }
            Command::LoadExample { example } => {
                let path = format!("examples/{}.txt", example);

                let lines = match util::read_lines(&path) {
                    Some(lines) => lines,
                    None => {
                        println!("file not found");
                        return;
                    }
                };

                self.proof_system = parse::proof_system(&lines);
                self.show_proof_system();
            }
            Command::Prove { prop } => {
                let proofs = prove::top_down(&self.proof_system, &prop);

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
                    print!("{}", unparse::proof(p, 1));
                    first = false;
                }
            }
            Command::Dualize => {
                self.proof_system = transform::dualize(&self.proof_system);
                self.show_proof_system();
            }
            Command::RemoveRule { rule } => {
                self.proof_system.retain(|r| r.name != rule);
                self.show_proof_system();
            }
            Command::RemovePremise { rule, premise } => {
                self.proof_system = self
                    .proof_system
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

                self.show_proof_system();
            }
            Command::AddAxiom { prop } => {
                self.proof_system.push(Rule::axiom(&prop));
                self.show_proof_system()
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
        }
    }
}

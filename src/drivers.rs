use crate::ao;
use crate::partition_navigation as pn;
use crate::pbn;

use ansi_term::Color::*;
use indexmap::IndexSet;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;

pub trait Driver<S: pbn::Step> {
    fn drive(&mut self, controller: pbn::Controller<S>) -> Option<S::Exp>;
}

////////////////////////////////////////////////////////////////////////////////
// CLI Driver

pub struct CliDriver;

impl CliDriver {
    pub fn new() -> Self {
        Self
    }
}

fn emit_graph(name: &str, e: &pn::Exp) {
    let highlights: HashMap<_, _> = e
        .partition()
        .iter()
        .map(|(oidx, c)| {
            (
                *oidx,
                match c {
                    pn::Class::Unseen => None,
                    pn::Class::Unknown => Some("gray"),
                    pn::Class::False => Some("red"),
                    // pastel = force_use is false
                    // saturated = force_use is true
                    // blue = undecided assume
                    pn::Class::True {
                        force_use: false,
                        assume: None,
                    } => Some("\"#CCFFFF\""),
                    pn::Class::True {
                        force_use: true,
                        assume: None,
                    } => Some("\"#55FFFF\""),
                    // green = decided not to assume
                    pn::Class::True {
                        force_use: false,
                        assume: Some(false),
                    } => Some("\"#CCFFCC\""),
                    pn::Class::True {
                        force_use: true,
                        assume: Some(false),
                    } => Some("green"),
                    // yellow = decided to assume
                    pn::Class::True {
                        force_use: false,
                        assume: Some(true),
                    } => Some("\"#FFFFCC\""),
                    pn::Class::True {
                        force_use: true,
                        assume: Some(true),
                    } => Some("yellow"),
                },
            )
        })
        .filter_map(|(oidx, c)| match c {
            Some(c) => Some((oidx, c.to_owned())),
            None => None,
        })
        .collect();

    let mut dot_file = File::create(format!("out/{}.dot", name)).unwrap();
    write!(&mut dot_file, "{}", e.graph().dot(&highlights)).unwrap();

    let pdf_file = File::create(format!("out/{}.pdf", name)).unwrap();
    let _ = Command::new("dot")
        .arg("-Tpdf")
        .arg(format!("out/{}.dot", name))
        .stdout(std::process::Stdio::from(pdf_file))
        .status()
        .unwrap();
}

impl Driver<pn::Step> for CliDriver {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<pn::Step>,
    ) -> Option<pn::Exp> {
        let mut round = 0;

        'main_loop: loop {
            let exp = controller.working_expression();

            emit_graph("interactive", &exp);

            round += 1;

            let valid = controller.valid();
            let mut options = controller.provide().unwrap();

            if !valid && options.is_empty() {
                println!("{}", Red.bold().paint("Not possible!"));
                return None;
            }

            let header = format!("══ Round {} {}", round, "═".repeat(40));

            println!(
                "{}\n\n{}\n\n    {}\n\n{}\n",
                Fixed(8).paint(header),
                Cyan.bold().paint("Working expression:"),
                exp,
                Cyan.bold().paint("Possible next steps:"),
            );

            for (i, option) in options.iter().cloned().enumerate() {
                let option_string =
                    Yellow.paint(format!("{}", option.show(&exp)));
                println!("  {}) {}", i + 1, option_string);
            }

            if controller.can_undo() {
                println!("{}", Fixed(8).paint("  u) undo"));
            }

            if valid {
                println!(
                    "  f) Expression is {}, finish navigation",
                    Green.bold().paint("valid")
                )
            }

            loop {
                print!(
                    "\n{} {}\n\n> ",
                    Purple.bold().paint("Which step would you like to take?"),
                    Fixed(8).paint("('q' to quit)"),
                );
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                if input == "q" {
                    return None;
                }

                if controller.can_undo() && input == "u" {
                    controller.undo();
                    break;
                }

                if valid && input == "f" {
                    break 'main_loop;
                }

                match input.parse::<usize>() {
                    Ok(choice) => {
                        if 1 <= choice && choice <= options.len() {
                            controller.decide(options.swap_remove(choice - 1));
                            break;
                        }
                    }
                    Err(_) => (),
                };
            }
        }

        let final_expression = controller.working_expression();

        println!(
            "\n{}\n\n    {}",
            Green.bold().paint("Final expression:"),
            final_expression
        );

        Some(final_expression)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Randomized goal-driven driver

pub struct RandomizedSolutionDrivenDriver {
    solution: IndexSet<ao::NodeId>,
    decisions: usize,
}

impl RandomizedSolutionDrivenDriver {
    pub fn new(solution: IndexSet<ao::NodeId>) -> Self {
        Self {
            solution,
            decisions: 0,
        }
    }

    pub fn decisions(&self) -> usize {
        self.decisions
    }
}

impl Driver<pn::Step> for RandomizedSolutionDrivenDriver {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<pn::Step>,
    ) -> Option<pn::Exp> {
        loop {
            let exp = controller.working_expression();
            if exp
                .filter_class(|c| {
                    c == pn::Class::True {
                        force_use: true,
                        assume: Some(true),
                    }
                })
                .ids(exp.graph())
                == self.solution
            {
                return Some(exp);
            }

            let mut options = controller.provide().unwrap();

            let idx = options
                .iter()
                .enumerate()
                .filter_map(|(i, option)| match &option {
                    pn::Step::SetClass(
                        id,
                        pn::Class::True {
                            force_use: true,
                            assume: Some(true),
                        },
                        _,
                    ) => {
                        if self.solution.contains(exp.graph().or_at(*id).id()) {
                            Some(i)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .choose(&mut rand::rng())?;

            self.decisions += idx;

            controller.decide(options.swap_remove(idx))
        }
    }
}

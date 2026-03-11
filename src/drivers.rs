use crate::partition_navigation as pn;
use crate::util;

use ansi_term::Color::*;
use aograph as ao;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub trait Driver<S: pbn::Step> {
    fn drive(
        &mut self,
        controller: pbn::Controller<util::Timer, S>,
    ) -> Option<S::Exp>;
}

////////////////////////////////////////////////////////////////////////////////
// CLI Driver

pub struct Cli {
    valid_word: String,
    pdf: bool,
}

impl Cli {
    pub fn new(valid_word: String, pdf: bool) -> Self {
        Self { valid_word, pdf }
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

impl Driver<pn::Step> for Cli {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<util::Timer, pn::Step>,
    ) -> Option<pn::Exp> {
        let mut round = 0;

        'main_loop: loop {
            round += 1;

            let valid = controller.valid();
            let mut options = controller.provide().ok()?;

            let exp = controller.working_expression();

            if self.pdf {
                emit_graph("INTERACTIVE", &exp);
            }

            if !valid && options.is_empty() {
                println!("{}", Red.bold().paint("Not possible!"));
                return None;
            }

            let header = format!("══ Round {} {}", round, "═".repeat(40));

            println!(
                "{}\n\n{}\n\n{}\n{}\n",
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
                    Green.bold().paint(&self.valid_word)
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

        let final_expression = controller.end();

        println!(
            "\n{}\n\n{}",
            Green.bold().paint("Final expression:"),
            final_expression
        );

        Some(final_expression)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Solution-driven driver

pub struct SolutionDriven {
    solution: pn::Exp,
    decisions: HashSet<(ao::OIdx, pn::Class)>,
    total_decisions: usize,
    latencies: Vec<u128>,
    count_unordered: bool,
}

impl SolutionDriven {
    pub fn new(solution: pn::Exp, count_unordered: bool) -> Self {
        Self {
            solution,
            decisions: HashSet::new(),
            total_decisions: 0,
            latencies: vec![],
            count_unordered,
        }
    }

    pub fn unique_decisions(&self) -> usize {
        self.decisions.len()
    }

    pub fn total_decisions(&self) -> usize {
        self.total_decisions
    }

    pub fn latencies(&self) -> &Vec<u128> {
        &self.latencies
    }
}

impl Driver<pn::Step> for SolutionDriven {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<util::Timer, pn::Step>,
    ) -> Option<pn::Exp> {
        loop {
            if controller.valid() {
                return Some(controller.end());
            }

            // println!("\nBEFORE! --------------------------------");

            let now = Instant::now();
            let mut options = controller.provide().ok()?;
            let latency = now.elapsed().as_millis();
            self.latencies.push(latency);

            let mut chosen_option = None;

            // println!(
            //     "{}",
            //     options
            //         .iter()
            //         .map(|o| o.show(controller.working_expression()))
            //         .collect::<Vec<_>>()
            //         .join("\n")
            // );

            for (i, option) in options.iter().enumerate() {
                match option {
                    pn::Step::SetClass(oidx, class, _) => {
                        self.total_decisions += 1;
                        let _ = self.decisions.insert((*oidx, *class));
                        // "User oracle" call
                        if self.solution.class(*oidx) == *class {
                            chosen_option = Some(i);
                            if !self.count_unordered {
                                break;
                            }
                        }
                    }
                    pn::Step::Seq(..) => {
                        panic!("SolutionDriver driver does not work with sequence steps");
                    }
                }
            }

            let chosen_option = chosen_option?;
            // .expect("SolutionDriven driver could not find consistent step");

            // println!("chosen option: {}", chosen_option);

            controller.decide(options.swap_remove(chosen_option))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Random driver

pub struct Random {
    go_until_maximal: bool,
}

impl Random {
    pub fn new(go_until_maximal: bool) -> Self {
        Self { go_until_maximal }
    }
}

impl Driver<pn::Step> for Random {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<util::Timer, pn::Step>,
    ) -> Option<pn::Exp> {
        loop {
            let e = controller.working_expression();

            let done = if self.go_until_maximal {
                e.maximal()
            } else {
                controller.valid()
            };

            if done {
                return Some(controller.end());
            }

            let mut options = controller.provide().ok()?;
            let choice = rand::rng().random_range(0..options.len());

            controller.decide(options.swap_remove(choice))
        }
    }
}

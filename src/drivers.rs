use crate::ao_navigation;
use crate::pbn;

use ansi_term::Color::*;
use rand::seq::IteratorRandom;
use std::io::Write;

pub trait Driver<S: pbn::Step> {
    fn drive(&mut self, controller: pbn::Controller<S>) -> Option<S::Exp>;
}

////////////////////////////////////////////////////////////////////////////////
// CLI Driver

pub struct CliDriver;

impl<S: std::fmt::Display + pbn::Step + Clone> Driver<S> for CliDriver
where
    S::Exp: std::fmt::Display,
{
    fn drive(
        &mut self,
        mut controller: pbn::Controller<S>,
    ) -> Option<<S as pbn::Step>::Exp> {
        let mut round = 0;

        loop {
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
                controller.working_expression(),
                Cyan.bold().paint("Possible next steps:"),
            );

            for (i, option) in options.iter().cloned().enumerate() {
                let option_string = Yellow.paint(format!("{}", option));
                println!("  {}) {}", i + 1, option_string);
            }

            if valid {
                println!(
                    "  f) Expression is {}, finish navigation",
                    Green.bold().paint("valid")
                )
            }

            let idx = loop {
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

                if valid && input == "f" {
                    break None;
                }

                match input.parse::<usize>() {
                    Ok(choice) => {
                        if 1 <= choice && choice <= options.len() {
                            break Some(choice - 1);
                        } else {
                            continue;
                        }
                    }
                    Err(_) => continue,
                };
            };

            let idx = match idx {
                Some(idx) => idx,
                None => break,
            };

            controller.decide(options.swap_remove(idx))
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
    solution: ao_navigation::AxiomSet,
    decisions: usize,
}

impl RandomizedSolutionDrivenDriver {
    pub fn new(solution: ao_navigation::AxiomSet) -> Self {
        Self {
            solution,
            decisions: 0,
        }
    }

    pub fn decisions(&self) -> usize {
        self.decisions
    }
}

impl Driver<ao_navigation::AOStep> for RandomizedSolutionDrivenDriver {
    fn drive(
        &mut self,
        mut controller: pbn::Controller<ao_navigation::AOStep>,
    ) -> Option<ao_navigation::AxiomSet> {
        loop {
            let exp = controller.working_expression();
            if exp == self.solution {
                return Some(exp);
            }

            let mut options = controller.provide().unwrap();

            let idx = options
                .iter()
                .enumerate()
                .filter_map(|(i, option)| match &option {
                    ao_navigation::AOStep::Add(label) => {
                        if self.solution.contains(label) {
                            Some(i)
                        } else {
                            None
                        }
                    }
                })
                .choose(&mut rand::rng())?;

            controller.decide(options.swap_remove(idx))
        }
    }
}

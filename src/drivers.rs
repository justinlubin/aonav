use crate::pbn;

use ansi_term::Color::*;
use std::io::Write;

pub struct CliDriver;

impl<S: std::fmt::Display + pbn::Step + Clone> pbn::Driver<S> for CliDriver
where
    S::Exp: std::fmt::Display,
{
    fn drive(
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

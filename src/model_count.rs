//! # Model Counting
//!
//! Counts models of Sat formulas

use rustsat::{instances::Cnf, types::Var};
use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

use crate::util::EarlyCutoff;

/// Returns None if unsat (0 models)
pub fn log10_model_count(
    n_vars: u32,
    cnf: &Cnf,
    projected_vars: Option<Vec<Var>>,
) -> Result<Option<f64>, EarlyCutoff> {
    let mut child = Command::new("ganak")
        .arg("--threads")
        .arg("1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Need to drop writer to send EOF for stdin
    {
        let mut child_stdin = child.stdin.take().unwrap();
        let mut writer = BufWriter::new(&mut child_stdin);

        write!(writer, "c t pmc\n").unwrap();

        cnf.write_dimacs(&mut writer, n_vars).unwrap();

        if let Some(pvs) = projected_vars {
            write!(writer, "c p show ").unwrap();
            for var in pvs {
                write!(writer, "{} ", var.to_ipasir()).unwrap();
            }
            write!(writer, "0\n").unwrap();
        }
    }

    let child_output =
        match child.wait_timeout(Duration::from_secs(30)).unwrap() {
            Some(_) => child.wait_with_output().unwrap(),
            None => {
                child.kill().unwrap();
                child.wait().unwrap();
                return Err(EarlyCutoff::TimerExpired);
            }
        };
    let result_string = String::from_utf8(child_output.stdout).unwrap();

    for line in result_string.lines() {
        if line == "s UNSATISFIABLE" {
            return Ok(None);
        }
        match line.strip_prefix("c s log10-estimate ") {
            Some(n) => return Ok(Some(n.parse::<f64>().unwrap())),
            None => (),
        }
    }

    panic!("Unexpected Ganak output format");
}

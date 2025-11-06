use rustsat::{instances::Cnf, types::Var};
use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
};

// Returns None if unsat (0 models)
pub fn log10_model_count(
    n_vars: u32,
    cnf: &Cnf,
    projected_vars: Option<Vec<Var>>,
) -> Option<f64> {
    let mut child = Command::new("ganak")
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

    let child_output = child.wait_with_output().unwrap();
    let result_string = String::from_utf8(child_output.stdout).unwrap();

    for line in result_string.lines() {
        if line == "s UNSATISFIABLE" {
            return None;
        }
        match line.strip_prefix("c s log10-estimate ") {
            Some(n) => return Some(n.parse::<f64>().unwrap()),
            None => (),
        }
    }

    panic!("Unexpected Ganak output format");
}

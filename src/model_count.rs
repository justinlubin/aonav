use rustsat::{instances::Cnf, types::Var};
use std::{
    io::{BufWriter, Write},
    process::{Command, Stdio},
};

pub fn model_count(
    n_vars: u32,
    cnf: &Cnf,
    projected_vars: Option<Vec<Var>>,
) -> usize {
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

        writer.flush().unwrap();
    }

    let child_output = child.wait_with_output().unwrap();
    let result_string = String::from_utf8(child_output.stdout).unwrap();

    for line in result_string.lines() {
        match line.strip_prefix("c s exact arb int ") {
            Some(n) => return n.parse::<usize>().unwrap(),
            None => (),
        }
    }

    panic!("Unexpected Ganak output format");
}

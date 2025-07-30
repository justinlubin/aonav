use crate::core::*;

#[derive(Debug, Clone)]
struct Context<'a> {
    lines: &'a Vec<String>,
    offset: usize,
    rules: Vec<Rule>,
}

impl<'a> Context<'a> {
    fn new(lines: &'a Vec<String>) -> Self {
        Context {
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

pub fn program(lines: &Vec<String>) -> Vec<Rule> {
    let mut p = Context::new(lines);
    p.program();
    p.rules
}

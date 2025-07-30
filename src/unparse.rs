use crate::core::*;

use colored::Colorize;

pub fn rule(rule: &Rule) -> String {
    let premises = rule.premises.join("   ");
    let conclusion = &rule.conclusion;
    let dash_count = std::cmp::max(premises.len(), conclusion.len()) + 2;
    format!(
        "  {: ^width$}\n  {} {}\n  {: ^width$}",
        premises.magenta(),
        "─".repeat(dash_count),
        rule.name.yellow(),
        conclusion.green(),
        width = dash_count,
    )
}

pub fn proof_system(rules: &ProofSystem) -> String {
    rules
        .iter()
        .map(|r| format!("{}\n{}", "rule:".bright_black(), rule(r)))
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn print_proof(proof: &Proof, indent: usize) {
    println!(
        "{} (by {})",
        proof.conclusion.green(),
        proof.rule_name.yellow()
    );
    for prem in &proof.premises {
        print!("{} • ", "  ".repeat(indent));
        print_proof(prem, indent + 1);
    }
}

use crate::{grammar, spellcheck};
use owo_colors::OwoColorize;

pub fn location(text: &str, offset: usize) -> (usize, usize) {
    let prefix = &text[..offset];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    (line, column)
}

pub fn print(spelling: &[spellcheck::Issue], grammar: &[grammar::Issue], color: bool) {
    for issue in spelling {
        let label = if color {
            "Spelling".red().to_string()
        } else {
            "Spelling".into()
        };
        let suggestions = if issue.suggestions.is_empty() {
            "no suggestions".to_string()
        } else {
            format!("suggestions: {}", issue.suggestions.join(", "))
        };
        println!(
            "{label} at {}:{}: '{}' ({suggestions})",
            issue.line, issue.column, issue.word
        );
    }
    for issue in grammar {
        let label = if color {
            "Grammar".yellow().to_string()
        } else {
            "Grammar".into()
        };
        println!(
            "{label} at {}:{}: {} [{}]",
            issue.line, issue.column, issue.message, issue.snippet
        );
    }
    println!(
        "{} spelling issues, {} grammar notes found",
        spelling.len(),
        grammar.len()
    );
}

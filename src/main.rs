mod cli;
mod config;
mod grammar;
mod report;
mod spellcheck;

use clap::Parser;
use cli::{Cli, Command};
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if let Some(Command::AddWord { word }) = cli.command {
        config::add_word(&word)?;
        println!("Added '{word}' to your personal dictionary.");
        return Ok(false);
    }

    let ignored = config::load()?;
    let checker = spellcheck::SpellChecker::new();
    let color = !cli.no_color && io::stdout().is_terminal();
    let check = |text: &str| {
        let spelling = checker.check(text, &ignored);
        let grammar = grammar::check(text);
        report::print(&spelling, &grammar, color);
        !spelling.is_empty() || !grammar.is_empty()
    };

    match cli.command {
        Some(Command::Check { file }) => Ok(check(&std::fs::read_to_string(file)?)),
        Some(Command::AddWord { .. }) => unreachable!(),
        None if !io::stdin().is_terminal() => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            Ok(check(&input))
        }
        None => {
            println!(
                "Enter text to check, one line at a time. Press Ctrl+D (Ctrl+Z on Windows) to exit."
            );
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            let mut found = false;
            loop {
                print!("> ");
                io::stdout().flush()?;
                match lines.next() {
                    Some(line) => {
                        let line = line?;
                        if !line.trim().is_empty() {
                            found |= check(&line);
                        }
                    }
                    None => {
                        println!();
                        break;
                    }
                }
            }
            Ok(found)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::from(1),
        Err(err) => {
            eprintln!("spellcheck: {err}");
            ExitCode::from(2)
        }
    }
}

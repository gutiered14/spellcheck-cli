use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "spellcheck", version, about = "Proofread text offline")]
pub struct Cli {
    #[arg(long, global = true, help = "Disable colored output")]
    pub no_color: bool,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check a UTF-8 text file
    Check { file: PathBuf },
    /// Ignore a word in future checks
    AddWord { word: String },
}

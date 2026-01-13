use anyhow::{Result, bail};
use clap::Parser;

/// Rust version of ‘grep’
#[derive(Debug, Parser)]
struct Args {
    /// Search pattern
    #[arg(required = true)]
    pattern: String,

    /// Input files(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

    /// Case-insensitive
    #[arg(short, long)]
    insensitive: bool,

    /// Recursive search
    #[arg(short, long)]
    recursive: bool,

    /// Count occurences
    #[arg(short, long)]
    count: bool,

    /// Invert match
    #[arg(short('v'), long)]
    invert: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(args);
    Ok(())
}

use anyhow::Result;
use clap::Parser;
use regex::{Regex, RegexBuilder};

/// Rust version of ‘fortune’
#[derive(Debug, Parser)]
#[command[about, author, version]]
struct CLIArgs {
    /// Input files or directories
    #[arg(value_name = "FILE", required = true)]
    sources: Vec<String>,

    /// Pattern
    #[arg(short = 'm', long)]
    pattern: Option<String>,

    /// Case-insensitive pattern matching
    #[arg(short, long)]
    insensitive: bool,

    /// Random seed
    #[arg(short, long)]
    seed: Option<u64>,
}

#[derive(Debug)]
struct Args {
    sources: Vec<String>,
    pattern: Option<Regex>,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    dbg!(args);
    Ok(())
}

fn parse_args() -> Result<Args> {
    let CLIArgs {
        sources,
        pattern,
        insensitive,
        ..
    } = CLIArgs::parse();

    let pattern = pattern
        .map(|pat| {
            RegexBuilder::new(pat.as_str())
                .case_insensitive(insensitive)
                .build()
        })
        .transpose()?;

    Ok(Args { sources, pattern })
}

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

/// Rust version of ’ls’
#[derive(Debug, Parser)]
#[command(author, about, version)]
struct CLIArgs {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,

    /// Long listing
    #[arg(short, long)]
    long: bool,

    /// Show all files
    #[arg(short = 'a', long = "all")]
    show_hidden: bool,
}

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    dbg!(args);
    Ok(())
}

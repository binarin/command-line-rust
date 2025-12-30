use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of ‘head’
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number of lines to print
    #[arg(
        value_name("LINES"),
        short('n'),
        long,
        default_value = "10",
        conflicts_with("bytes")
    )]
    lines: u64,

    /// Number of bytes to print
    #[arg(
        value_name("BYTES"),
        short('c'),
        long,
    )]
    bytes: Option<u64>,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    dbg!(args);
    Ok(())
}

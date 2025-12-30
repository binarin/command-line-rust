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
    #[arg(value_name("lines"), short('n'), long("lines"), default_value_t = 10)]
    lines: u64,

    /// Number of bytes to print
    #[arg(
        value_name("bytes"),
        short('c'),
        long("bytes"),
        conflicts_with("lines")
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

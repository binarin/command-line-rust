use clap::Parser;
use anyhow::Result;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// ‘wc’ in Rust
struct Args {
    #[arg(value_name="FILE", default_value="-")]
    /// filenames (or ‘-’ for stdin)
    files: Vec<String>,

    #[arg(short, long, default_value_t = true)]
    /// print the newline counts
    lines: bool,

    #[arg(short, long, default_value_t = true)]
    /// print the word count
    words: bool,

    #[arg(short('c'), long, default_value_t = true)]
    /// print the bytes count
    bytes: bool,

    #[arg(short('m'), long, default_value_t = true)]
    /// print the characters count
    chars: bool,
}

fn main() {
    run(Args::parse()).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
}

fn run(args: Args) -> Result<()> {
    dbg!(args);
    Ok(())
}

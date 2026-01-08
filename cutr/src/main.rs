// Things to do:
// • Read and write a delimited text file using the csv crate
// • Deference a value using *
// • Use Iterator::flatten to remove nested structures from iterators
// • Use Iterator::flat_map to combine Iterator::map and Iterator::flatten

use anyhow::Result;
use clap::{Args as ClapArgs, Parser};

/// Rust version of ‘cut’
#[derive(Debug, Parser)]
#[command(about, author, version)]
struct Args {
    /// Inputs files(s)
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(short, long, default_value = "\t")]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

#[derive(Debug, Clone, ClapArgs)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    /// Selected fields
    #[arg(short, long)]
    fields: Option<String>,

    /// Selected bytes
    #[arg(short, long)]
    bytes: Option<String>,

    /// Selected chars
    #[arg(short, long)]
    chars: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(args);
    Ok(())
}

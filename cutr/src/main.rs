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
    #[arg(short, long, default_value = "\t", value_parser = parse_delimiter)]
    delimiter: u8,

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

fn parse_delimiter(s: &str) -> Result<u8, String> {
    match s.len() {
        1 => s.as_bytes().first().map_or(Err("must be a single byte".to_string()), |b| Ok(*b)),
        _ => Err("must be a single byte".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn delimiter_value_parser() {
        assert_eq!(Ok(46), parse_delimiter("."));
        assert_eq!(
            Err("must be a single byte".to_string()),
            parse_delimiter(",,")
        );
    }
}

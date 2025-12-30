use std::{fs::File, io::{self, BufRead, BufReader}};

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
        value_parser = clap::value_parser!(u64).range(1..),
        conflicts_with("bytes")
    )]
    lines: u64,

    /// Number of bytes to print
    #[arg(
        value_name("BYTES"),
        short('c'),
        long,
        value_parser = clap::value_parser!(u64).range(1..),
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
    for filename in args.files {
        match open(&filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(_) => println!("Opened {filename}"),
        }
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

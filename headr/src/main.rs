use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

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
    let multifile = args.files.len() > 1;
    for (file_no, filename) in args.files.iter().enumerate() {
        if multifile {
            if file_no > 0 {
                print!("\n");
            }
            println!("==> {filename} <==");
        }
        match open(&filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => process_file(file, args.lines, args.bytes).unwrap_or(()),
        }
    }
    Ok(())
}

fn process_file(mut file: Box<dyn BufRead>, mut lines: u64, bytes: Option<u64>) -> Result<()> {
    while lines > 0 {
        let mut s = String::new();
        let bytes_read = file.read_line(&mut s)?;
        if bytes_read == 0 {
            break;
        }
        print!("{s}");
        lines -= 1;
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

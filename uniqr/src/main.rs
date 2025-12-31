use std::{fs::File, io::{self, BufRead, BufReader}};

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// ‘uniq’ in Rust - omit repeated lines
struct Args {
    #[arg(value_name("INPUT"), default_value = "-")]
    in_file: String,

    #[arg(value_name("OUTPUT"))]
    out_file: Option<String>,

    #[arg(short, long)]
    /// prefix lines by the number of occurences
    count: bool,
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

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

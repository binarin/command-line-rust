use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use anyhow::Result;
use anyhow::anyhow;
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
    let file = open(&args.in_file).map_err(|err| anyhow!("{}: {err}", args.in_file))?;
    let mut out = open_out_file(&args)?;

    let mut prev_line = String::new();
    // It will be ‘0’ at start of every unique group
    let mut prev_count = 0;

    for line_result in file.lines() {
        let line = line_result?;

        if prev_count > 0 {
            if line == prev_line {
                prev_count += 1;
            } else {
                if args.count {
                    write!(out, "{prev_count:>7} {prev_line}\n")?;
                } else {
                    write!(out, "{prev_line}\n")?;
                }
                prev_count = 0; // Make code below start the new unique group
            }
        }

        if prev_count == 0 {
            prev_line = line;
            prev_count = 1;
        }
    }

    if prev_count > 0 {
        if args.count {
            write!(out, "{prev_count:>7} {prev_line}\n")?;
        } else {
            write!(out, "{prev_line}\n")?;
        }
    }

    Ok(())
}

fn open_out_file(args: &Args) -> Result<Box<dyn Write>> {
    if let Some(filename) = &args.out_file {
        return Ok(Box::new(File::create(filename)?));
    } else {
        return Ok(Box::new(std::io::stdout()));
    }
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

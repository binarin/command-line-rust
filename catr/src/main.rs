use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Rust version of cat ‘cat’
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number lines
    #[arg(short('n'), long("number"), conflicts_with("number_nonblank_lines"))]
    number_lines: bool,

    /// Number non-blank lines
    #[arg(short('b'), long("number-nonblank"))]
    number_nonblank_lines: bool,
}

fn run(args: Args) -> Result<()> {
    for filename in args.files {
        match open(&filename) {
            Err(err) => {
                eprintln!("Failed to open {filename}: {err}");
            }
            Ok(file) => print_file(file, args.number_lines, args.number_nonblank_lines)?,
        }
    }
    Ok(())
}

fn print_file(
    file: Box<dyn BufRead>,
    number_lines: bool,
    number_nonblank_lines: bool,
) -> Result<()> {
    let mut ctr: u32 = 1;
    for line_res in file.lines() {
        let line = line_res?;
        if number_lines || (number_nonblank_lines && !line.is_empty()) {
            print!("{ctr:6}\t");
            ctr += 1;
        }
        println!("{line}");
    }
    Ok(())
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(0);
    }
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

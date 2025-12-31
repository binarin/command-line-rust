use std::{fs::File, io::{BufRead, BufReader}};

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// ‘wc’ in Rust
struct Args {
    #[arg(value_name = "FILE", default_value = "-")]
    /// filenames (or ‘-’ for stdin)
    files: Vec<String>,

    #[arg(short, long, default_value_t = false)]
    /// print the newline counts
    lines: bool,

    #[arg(short, long, default_value_t = false)]
    /// print the word count
    words: bool,

    #[arg(short('c'), long, default_value_t = false)]
    /// print the bytes count
    bytes: bool,

    #[arg(short('m'), long, default_value_t = false, conflicts_with("bytes"))]
    /// print the characters count
    chars: bool,
}

#[derive(Debug, PartialEq, Default)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

fn main() {
    run(parse_args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    });
}

fn run(args: Args) -> Result<()> {
    for filename in &args.files {
        match open(&filename) {
            Ok(file) => {
                count(file, &args);
                ()
            }
            Err(err) => eprintln!("{err}"),
        }
    }
    Ok(())
}

fn count(file: Box<dyn BufRead>, args: &Args) -> FileInfo {
    FileInfo::default()
}

fn parse_args() -> Args {
    let mut args = Args::parse();

    // none of the explicit args is present
    if [args.lines, args.words, args.bytes, args.chars]
        .iter()
        .all(|v| !v)
    {
        args.lines = true;
        args.words = true;
        args.bytes = true;
    }
    args
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

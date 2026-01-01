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
    let mut file = open(&args.in_file).map_err(|err| anyhow!("{}: {err}", args.in_file))?;

    let mut out = open_out_file(&args)?;

    let mut prev_line: Option<Vec<u8>> = None;
    let mut prev_eol: Vec<u8> = vec!();
    let mut prev_count = 1;

    loop {
        let mut line = String::new();
        let bytes_read = file.read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }

        let eol_byte_count = line
            .bytes()
            .rev()
            .take_while(|c| *c == b'\n' || *c == b'\r')
            .count();
        let line_bytes = line.as_bytes();
        let line_without_eol = &line_bytes[0..line.len() - eol_byte_count];

        if let Some(ref pl) = prev_line {
            if pl == line_without_eol {
                prev_count += 1;
            } else {
                if args.count {
                    out.write(format!("{:>4} ", prev_count).as_bytes())?;
                }
                out.write(&pl)?;
                out.write(&prev_eol)?;

                prev_line = None;
            }
        }

        if prev_line.is_none() {
            prev_line = Some(line_without_eol.to_vec());
            prev_eol = line_bytes[(line.len() - eol_byte_count)..].to_vec();
            prev_count = 1;
        }
    }

    if let Some(prev_line) = prev_line {
        if args.count {
            out.write(format!("{:>4} ", prev_count).as_bytes())?;
        }
        out.write(&prev_line)?;
        out.write(&prev_eol)?;
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

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

use anyhow::{Result, anyhow};

use clap::Parser;

// As in GNU uniq
const COUNT_FIELD_WIDTH: usize = 7;

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

fn write_line(
    out: &mut dyn std::io::Write,
    line: &str,
    count: usize,
    show_count: bool,
) -> Result<()> {
    if show_count {
        writeln!(out, "{count:>width$} {line}", width = COUNT_FIELD_WIDTH)?;
    } else {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

fn run(args: Args) -> Result<()> {
    let file = open(&args.in_file).map_err(|err| anyhow!("{}: {err}", args.in_file))?;
    let mut out = open_out_file(&args)?;

    let mut previous: Option<(String, usize)> = None;

    for line_result in file.lines() {
        let line = line_result?;

        if let Some((prev_line, prev_count)) = &mut previous {
            if prev_line == &line {
                *prev_count += 1;
                continue;
            }
            write_line(out.as_mut(), prev_line, *prev_count, args.count)?;
        }
        previous = Some((line, 1));
    }

    if let Some((line, count)) = previous {
        write_line(&mut out, &line, count, args.count)?;
    }

    Ok(())
}

fn open_out_file(args: &Args) -> Result<Box<dyn Write>> {
    match &args.out_file {
        Some(filename) => Ok(Box::new(File::create(filename)?)),
        None => Ok(Box::new(std::io::stdout())),
    }
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

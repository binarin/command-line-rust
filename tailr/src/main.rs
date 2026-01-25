use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Debug, Clone, PartialEq)]
enum Pos {
    FromStart(usize),
    FromEnd(usize),
}

#[derive(Debug)]
enum Mode {
    Lines(Pos),
    Bytes(Pos),
}

/// Rust version of ‘tail’
#[derive(Debug, Parser)]
#[command(about, author, version)]
struct CLIArgs {
    /// Input file(s)
    #[arg(value_name = "FILE", required = true)]
    files: Vec<String>,

    /// Number of lines
    #[arg(short('n'), long, value_parser=parse_pos, default_value = "10")]
    lines: Pos,

    /// Number of bytes
    #[arg(short('c'), long, value_parser=parse_pos, conflicts_with("lines"))]
    bytes: Option<Pos>,

    /// Suppress headers
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug)]
struct Args {
    files: Vec<String>,
    quiet: bool,
    mode: Mode,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    for file in args.files.clone() {
        match File::open(&file) {
            Ok(mut fh) => process_file(&file, &args, &mut fh),
            Err(e) => eprintln!("{file}: {e}"),
        }
    }
    Ok(())
}

fn process_file(file: &str, args: &Args, fh: &mut File) {
    if !args.quiet && args.files.len() > 1 {
        println!("{file}:");
    }

    match &args.mode {
        Mode::Lines(_) => todo!(),
        Mode::Bytes(pos) => match process_file_bytes(file, pos, fh) {
            Ok(_) => (),
            Err(e) => eprintln!("{file}: {e}"),
        },
    }
}

fn process_file_bytes(file: &str, start_target: &Pos, fh: &mut File) -> Result<()> {
    fh.seek(SeekFrom::End(0))
        .map_err(|e| anyhow!("{file} - while seeking to the end: {e}"))?;

    let len: usize = fh.stream_position()?.try_into()?;

    let pos = match &start_target {
        Pos::FromStart(offset) => std::cmp::min(len, *offset),
        Pos::FromEnd(negative_offset) => {
            if *negative_offset > len {
                0
            } else {
                len - *negative_offset
            }
        }
    };

    fh.seek(SeekFrom::Start(pos.try_into()?))?;

    let mut output = std::io::stdout();

    let mut buf = [0 as u8; 4096];
    loop {
        let bytes_read = fh.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        output.write_all(&buf[0..bytes_read])?;
    }

    Ok(())
}

fn parse_args() -> Result<Args> {
    let CLIArgs {
        files,
        lines,
        bytes,
        quiet,
    } = CLIArgs::parse();

    let mode = if bytes.is_some() {
        Mode::Bytes(bytes.unwrap())
    } else {
        Mode::Lines(lines)
    };

    Ok(Args { files, mode, quiet })
}

fn parse_pos(arg: &str) -> Result<Pos> {
    if arg.is_empty() {
        return Err(anyhow!("Position arg can't be empty"));
    }
    let (from_start, num) = match arg.chars().nth(0) {
        Some('+') => (true, &arg[1..]),
        Some('-') => (false, &arg[1..]),
        _ => (false, arg),
    };
    let num: usize = num.parse().map_err(|err| anyhow!("{arg}: {err}"))?;

    match from_start {
        true => Ok(Pos::FromStart(if num > 0 { num - 1 } else { 0 })), // ‘+n’ are one-base indexed (and ‘+0’ is an exception)
        false => Ok(Pos::FromEnd(num)),
    }
}

#[cfg(test)]
mod tests {
    use super::Pos::*;
    use super::*;
    use assertables::*;

    #[test]
    fn test_parse_pos() {
        // no prefix -> from end
        let res = parse_pos("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromEnd(3));

        // leading "+"
        let res = parse_pos("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromStart(2));

        // An explicit "-" prefix is the same as no prefix
        let res = parse_pos("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromEnd(3));

        // Zero is zero
        let res = parse_pos("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromEnd(0));

        // Plus zero is special
        let res = parse_pos("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromStart(0));

        // Test boundaries
        let res = parse_pos(format!("+{}", usize::MAX).as_str());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromStart(usize::MAX - 1));

        // A floating-point value is invalid
        let res = parse_pos("3.14");
        assert!(res.is_err());
        assert_contains!(
            res.unwrap_err().to_string(),
            "invalid digit found in string"
        );

        // Any non-integer string is invalid
        let res = parse_pos("foo");
        assert!(res.is_err());
        assert_contains!(
            res.unwrap_err().to_string(),
            "invalid digit found in string"
        );
    }
}

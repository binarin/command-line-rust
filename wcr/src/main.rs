use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

#[derive(Debug, PartialEq, Default, Copy, Clone)]
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
    let mut totals = FileInfo::default();

    for filename in &args.files {
        open(filename)
            .and_then(|file| {
                let fi = count(file)?;
                totals.num_lines += fi.num_lines;
                totals.num_words += fi.num_words;
                totals.num_bytes += fi.num_bytes;
                totals.num_chars += fi.num_chars;
                let filename_part: String = if filename == "-" && args.files.len() == 1 {
                    "".to_string()
                } else {
                    " ".to_string() + filename
                };
                println!("{}{}", render_file_info(&fi, &args), filename_part);
                Ok(())
            })
            .unwrap_or_else(|err| eprintln!("{filename}: {err}"));
    }
    if args.files.len() > 1 {
        println!("{} total", render_file_info(&totals, &args));
    }
    Ok(())
}

fn render_file_info(fi: &FileInfo, args: &Args) -> String {
    let mut ret = " ".to_string();
    if args.lines {
        ret += &format!("{:>7} ", fi.num_lines).to_string();
    }
    if args.words {
        ret += &format!("{:>7} ", fi.num_words).to_string();
    }
    if args.chars {
        ret += &format!("{:>7} ", fi.num_chars).to_string();
    }
    if args.bytes {
        ret += &format!("{:>7} ", fi.num_bytes).to_string();
    }
    ret.trim_end().to_string()
}

fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_chars = 0;
    let mut num_bytes = 0;
    loop {
        let mut buf = String::new();
        let bytes_read = file.read_line(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_words += buf.split_whitespace().count();
        num_lines += 1;
        num_chars += buf.chars().count();
        num_bytes += bytes_read;
    }
    Ok(FileInfo {
        num_lines,
        num_words,
        num_chars,
        num_bytes,
    })
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

#[cfg(test)]
mod tests {
    use assertables::*;

    use super::{FileInfo, count};
    use std::io::Cursor;

    fn assert_count_string(
        s: &str,
        num_lines: usize,
        num_words: usize,
        num_chars: usize,
        num_bytes: usize,
    ) {
        let expected = FileInfo {
            num_lines,
            num_words,
            num_chars,
            num_bytes,
        };
        assert_ok_eq_x!(count(Cursor::new(s)), expected);
    }

    #[test]
    fn test_count_empty() {
        assert_count_string("", 0, 0, 0, 0);
    }

    #[test]
    fn test_count() {
        assert_count_string(
            "I don't want the world.\nI just want your half.\r\n",
            2,
            10,
            48,
            48,
        );
    }
}

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    let mut need_newline_before = false;

    for file in &args.files {
        _ = process_file(file, &args, &mut need_newline_before)
            .map_err(|e| eprintln!("{file}: {e}"));
    }
    Ok(())
}

fn process_file(file: &str, args: &Args, need_newline_before: &mut bool) -> Result<()> {
    let mut fh = File::open(file)?;

    if !args.quiet && args.files.len() > 1 {
        if *need_newline_before {
            println!();
        }
        println!("==> {file} <==");
        *need_newline_before = true;
    }

    let seek_pos = match &args.mode {
        Mode::Lines(pos) => lines_seek_pos(pos, &mut fh)?,
        Mode::Bytes(pos) => bytes_seek_pos(pos, &mut fh)?,
    };

    copy_to_stdout(&mut fh, &seek_pos)?;

    Ok(())
}

fn bytes_seek_pos(pos: &Pos, fh: &mut File) -> Result<SeekFrom> {
    fh.seek(SeekFrom::End(0))?;

    let len: usize = fh.stream_position()?.try_into()?;

    // NOTE: SeekFrom::Start(u64), but SeekFrom::End(i64)
    match pos {
        Pos::FromStart(offset) => Ok(SeekFrom::Start(std::cmp::min(len, *offset).try_into()?)),
        Pos::FromEnd(offset) => Ok(SeekFrom::End(-std::cmp::min(len, *offset).try_into()?)),
    }
}

fn lines_seek_pos(pos: &Pos, fh: &mut File) -> Result<SeekFrom> {
    match pos {
        Pos::FromStart(offset) => {
            let mut buf = [0_u8; 4096];
            let mut rem = *offset;
            let mut skip_byte: usize = 0;
            'outer: loop {
                if rem == 0 {
                    break;
                }
                let bytes_read = fh.read(&mut buf)?;
                if bytes_read == 0 {
                    break;
                }
                for byte in &buf[0..bytes_read] {
                    skip_byte += 1;
                    if *byte == b'\n' {
                        rem -= 1;
                        if rem == 0 {
                            break 'outer;
                        }
                    }
                }
            }
            Ok(SeekFrom::Start(skip_byte.try_into()?))
        }
        Pos::FromEnd(0) => Ok(SeekFrom::End(0)),
        Pos::FromEnd(offset) => {
            let mut scanner = BackScanner::new(fh)?;
            let mut need_bytes: i64 = 0;

            let mut rem = *offset;

            if let Some(b'\n') = scanner.peek() {
                // to show last line -> we need to find 2nd newline from end
                rem += 1;
            }

            for byte in scanner {
                let byte = byte?;
                if byte == b'\n' {
                    rem -= 1;
                    if rem == 0 {
                        break;
                    }
                }
                need_bytes += 1;
            }

            Ok(SeekFrom::End(-need_bytes))
        }
    }
}

const BUF_SIZE: usize = if cfg!(test) { 10 } else { 4_096 };

struct BackScanner<'a, FH> {
    fh: &'a mut FH,
    buf: [u8; BUF_SIZE],
    buf_pos: usize,
    buf_offset_in_file: usize,
}

impl<'a, FH: Seek + Read> BackScanner<'a, FH> {
    fn new(fh: &'a mut FH) -> Result<Self> {
        fh.seek(SeekFrom::End(0))?;
        let file_len: usize = fh.stream_position()?.try_into()?;

        let mut last_chunk_len = file_len % BUF_SIZE;

        if last_chunk_len == 0 && file_len >= BUF_SIZE {
            last_chunk_len = BUF_SIZE;
        }

        let buf_offset_in_file: usize = file_len.saturating_sub(last_chunk_len);
        let buf = [0_u8; BUF_SIZE];

        let mut scanner = BackScanner {
            fh,
            buf,
            buf_pos: BUF_SIZE,
            buf_offset_in_file,
        };

        Self::fill_buf(&mut scanner)?;

        Ok(scanner)
    }

    fn fill_buf(&mut self) -> Result<()> {
        let mut buf_target: usize = 0;
        self.fh
            .seek(SeekFrom::Start(self.buf_offset_in_file.try_into()?))?;
        loop {
            let bytes_read = self.fh.read(&mut self.buf[buf_target..])?;
            buf_target += bytes_read;
            if buf_target == BUF_SIZE || bytes_read == 0 {
                break;
            }
        }
        self.buf_pos = buf_target;
        Ok(())
    }

    fn peek(&mut self) -> Option<u8> {
        if self.buf_pos > 0 {
            Some(self.buf[self.buf_pos - 1])
        } else {
            None
        }
    }
}

impl<'a, FH: Seek + Read> Iterator for BackScanner<'a, FH> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf_pos == 0 {
            if self.buf_offset_in_file == 0 {
                return None;
            }

            self.buf_offset_in_file -= BUF_SIZE;
            assert!(self.buf_offset_in_file.is_multiple_of(BUF_SIZE));

            if let Err(e) = self.fill_buf() {
                return Some(Err(e));
            }
        }

        self.buf_pos -= 1;

        Some(Ok(self.buf[self.buf_pos]))
    }
}

fn copy_to_stdout(fh: &mut File, seek: &SeekFrom) -> Result<()> {
    fh.seek(*seek)?;

    let mut output = std::io::stdout();

    let mut buf = [0_u8; 4096];
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

    let mode = if let Some(bytes) = bytes {
        Mode::Bytes(bytes)
    } else {
        Mode::Lines(lines)
    };

    Ok(Args { files, mode, quiet })
}

fn parse_pos(arg: &str) -> Result<Pos> {
    if arg.is_empty() {
        return Err(anyhow!("Position arg can't be empty"));
    }
    let (from_start, num) = match arg.chars().next() {
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
    use assertables::*;
    use learnr::assert_err_str_contains;

    use super::Pos::*;
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_pos() {
        // no prefix -> from end
        assert_ok_eq_x!(parse_pos("3"), FromEnd(3));

        // leading "+"
        assert_ok_eq_x!(parse_pos("+3"), FromStart(2));

        // An explicit "-" prefix is the same as no prefix
        assert_ok_eq_x!(parse_pos("-3"), FromEnd(3));

        // Zero is zero
        assert_ok_eq_x!(parse_pos("0"), FromEnd(0));

        // Plus zero is special
        assert_ok_eq_x!(parse_pos("+0"), FromStart(0));

        // Test boundaries
        assert_ok_eq_x!(
            parse_pos(format!("+{}", usize::MAX).as_str()),
            FromStart(usize::MAX - 1)
        );

        // A floating-point value is invalid
        assert_err_str_contains!(parse_pos("3.14"), "invalid digit found in string");

        // Any non-integer string is invalid
        assert_err_str_contains!(parse_pos("foo"), "invalid digit found in string");
    }

    #[test]
    fn backscanner_empty_file() -> Result<()> {
        let mut fh = Cursor::new("");
        assert_eq!(None, BackScanner::new(&mut fh)?.peek());
        if BackScanner::new(&mut fh)?.next().is_some() {
            panic!("Should never get here");
        }
        Ok(())
    }

    #[test]
    fn backscanner_small_file() -> Result<()> {
        let mut fh = Cursor::new("abcdef");
        assert_eq!(
            "fedcba".to_string(),
            BackScanner::new(&mut fh)?
                .map(|r| -> char { r.unwrap().into() })
                .collect::<String>()
        );
        Ok(())
    }
    #[test]

    // big -> more that BUF_SIZE
    fn backscanner_big_file() -> Result<()> {
        let contents = "012345678901234567890123456789XXX".to_string();
        let mut fh = Cursor::new(&contents);
        assert_eq!(
            contents.chars().rev().collect::<String>(),
            BackScanner::new(&mut fh)?
                .map(|r| -> char { r.unwrap().into() })
                .collect::<String>()
        );
        Ok(())
    }
}

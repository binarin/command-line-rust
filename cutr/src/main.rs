use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Range;

use anyhow::Result;
use anyhow::bail;
use clap::{Args as ClapArgs, Parser};

/// Rust version of ‘cut’
#[derive(Debug, Parser)]
#[command(about, author, version)]
struct Args {
    /// Inputs files(s)
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(short, long, default_value = "\t", value_parser = parse_delimiter)]
    delimiter: u8,

    #[command(flatten)]
    extract: ArgsExtract,
}

#[derive(Debug, Clone, ClapArgs)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    /// Selected fields
    #[arg(short, long, value_parser = parse_pos)]
    fields: Option<PositionList>,

    /// Selected bytes
    #[arg(short, long, value_parser = parse_pos)]
    bytes: Option<PositionList>,

    /// Selected chars
    #[arg(short, long, value_parser = parse_pos)]
    chars: Option<PositionList>,
}

type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

fn main() -> Result<()> {
    run(Args::parse())
}

fn run(args: Args) -> Result<()> {
    let extract = build_extract(&args.extract)?;
    args.files.iter().for_each(|filename| match open(filename) {
        Err(e) => eprintln!("{filename}: {e}"),
        Ok(mut file) => extract_file(filename, &mut file, &extract, &args),
    });
    Ok(())
}

fn build_extract(args: &ArgsExtract) -> Result<Extract> {
    match args {
        ArgsExtract {
            fields: Some(fs), ..
        } => Ok(Extract::Fields(fs.clone())),
        ArgsExtract {
            chars: Some(cs), ..
        } => Ok(Extract::Chars(cs.clone())),
        ArgsExtract {
            bytes: Some(bs), ..
        } => Ok(Extract::Bytes(bs.clone())),
        _ => unreachable!("clap must ensure that there is exactly one option set in '{args:?}'"),
    }
}

fn extract_file(filename: &str, file: &mut impl BufRead, extract: &Extract, args: &Args) {
    match extract {
        Extract::Chars(pl) => file.lines().for_each(|line| match line {
            Err(e) => eprintln!("{filename}: bad line {e}"),
            Ok(line) => println!("{}", extract_chars(&line, pl)),
        }),
        Extract::Bytes(bl) => file.lines().for_each(|line| match line {
            Err(e) => eprintln!("{filename}: bad line {e}"),
            Ok(line) => println!("{}", extract_bytes(&line, bl)),
        }),
        Extract::Fields(fl) => extract_fields_from_file(file, fl, args.delimiter),
    }
}

fn parse_single_position(s: &str) -> Result<usize> {
    let mut result: usize = 0;
    for c in s.chars() {
        match c.to_digit(10) {
            Some(val) => result = result * 10 + val as usize,
            None => bail!("Invalid char {c}"),
        }
    }
    if result == 0 {
        bail!("Should be positive");
    }
    Ok(result)
}

fn parse_pos(pos: &str) -> Result<PositionList> {
    pos.split(',')
        .map(|range| match range.split_once('-') {
            Some((fst, snd)) => {
                let start = parse_single_position(fst)?;
                let end = parse_single_position(snd)?;
                if start >= end {
                    bail!(
                        "First number in range ({start}) must be lower than second number ({end})"
                    );
                }
                Ok(Range {
                    start: start - 1,
                    end,
                })
            }
            _ => Ok(parse_single_position(range).map(|start| Range {
                start: start - 1,
                end: start,
            })?),
        })
        .collect::<Result<PositionList>>()
        .and_then(|lst| match lst.len() {
            0 => bail!("empty pos list"),
            _ => Ok(lst),
        })
}

fn extract_fields_from_file(file: &mut impl BufRead, fields_pos: &PositionList, delimiter: u8) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(file);

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(std::io::stdout());

    for line in rdr.records() {
        match line {
            Ok(line) => {
                let _ = wtr.write_record(extract_fields(&line, fields_pos));
            }
            Err(e) => eprintln!("{e}"),
        }
    }

    let _ = wtr.flush();
}

fn extract_fields(line: &csv::StringRecord, fields_pos: &[Range<usize>]) -> Vec<String> {
    let mut result = Vec::new();
    for Range { start, end } in fields_pos {
        let mut subfields: Vec<String> = line
            .iter()
            .skip(*start)
            .take(end - start)
            .map(From::from)
            .collect();
        result.append(&mut subfields);
    }
    result
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let mut result = String::new();
    for Range { start, end } in char_pos {
        let substr: String = line.chars().skip(*start).take(end - start).collect();
        result += &substr;
    }
    result
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut result = String::new();
    for Range { start, end } in byte_pos {
        let subbytes = line
            .bytes()
            .skip(*start)
            .take(end - start)
            .collect::<Vec<u8>>();
        result += &String::from_utf8_lossy(&subbytes);
    }
    result
}

fn parse_delimiter(s: &str) -> Result<u8, String> {
    match s.len() {
        1 => s
            .as_bytes()
            .first()
            .map_or(Err("must be a single byte".to_string()), |b| Ok(*b)),
        _ => Err("must be a single byte".to_string()),
    }
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::single_range_in_vec_init)]
    use assertables::*;
    use csv::StringRecord;
    use learnr::assert_err_str_contains;

    use crate::*;

    #[test]
    fn delimiter_value_parser() {
        assert_eq!(Ok(46), parse_delimiter("."));
        assert_eq!(
            Err("must be a single byte".to_string()),
            parse_delimiter(",,")
        );
    }

    fn test_parse_pos(s: &str, exp: Vec<(usize, usize)>) {
        let pr = parse_pos(s).unwrap();
        assert_eq!(
            exp.iter()
                .map(|(start, end)| Range {
                    start: *start,
                    end: *end
                })
                .collect::<PositionList>(),
            pr
        );
    }

    #[test]
    fn parse_pos_single() {
        test_parse_pos("5", vec![(4, 5)]);
        test_parse_pos("5,1", vec![(4, 5), (0, 1)]);
    }

    #[test]
    fn parse_pos_range() {
        test_parse_pos("9-15", vec![(8, 15)]);
        test_parse_pos("9-15,14-31,8", vec![(8, 15), (13, 31), (7, 8)]);
    }

    #[test]
    fn test_parse_pos_from_book() {
        // The empty string is an error
        assert_err!(parse_pos(""));

        // Zero is an error
        assert_err_str_contains!(parse_pos("0"), r#"Should be positive"#);
        assert_err_str_contains!(parse_pos("0-1"), r#"Should be positive"#);

        // A leading "+" is an error
        assert_err_str_contains!(parse_pos("+1"), r#"Invalid char +"#);
        assert_err_str_contains!(parse_pos("+1-2"), r#"Invalid char +"#);
        assert_err_str_contains!(parse_pos("1-+2"), r#"Invalid char +"#);

        // Any non-number is an error
        assert_err_str_contains!(parse_pos("a"), r#"Invalid char a"#);
        assert_err_str_contains!(parse_pos("1,a"), r#"Invalid char a"#);
        assert_err_str_contains!(parse_pos("1-a"), r#"Invalid char a"#);
        assert_err_str_contains!(parse_pos("a-1"), r#"Invalid char a"#);

        // Wonky ranges
        assert_err!(parse_pos("-"));
        assert_err!(parse_pos(","));
        assert_err!(parse_pos("1,"));
        assert_err!(parse_pos("1-"));
        assert_err!(parse_pos("1-1-1"));
        assert_err!(parse_pos("1-1-a"));

        // First number must be less than second
        assert_err_str_contains!(
            parse_pos("1-1"),
            "First number in range (1) must be lower than second number (1)"
        );
        assert_err_str_contains!(
            parse_pos("2-1"),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(
            extract_chars("", &[Range { start: 0, end: 1 }]),
            "".to_string()
        );
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }
    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}

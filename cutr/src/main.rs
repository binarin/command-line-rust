// Things to do:
// • Read and write a delimited text file using the csv crate
// • Deference a value using *
// • Use Iterator::flatten to remove nested structures from iterators
// • Use Iterator::flat_map to combine Iterator::map and Iterator::flatten

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
        _ => panic!("clap must ensure that there is exactly one option set in '{args:?}'"),
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
    if result <= 0 {
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
                    end: end,
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

fn main() -> Result<()> {
    let args = Args::parse();
    let extract = build_extract(&args.extract)?;
    dbg!(extract);
    Ok(())
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

#[cfg(test)]
mod tests {
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
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Should be positive"#);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Should be positive"#);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char +"#,);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char +"#,);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char +"#,);
        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char a"#);
        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char a"#);
        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char a"#,);
        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid char a"#,);
        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());
        let res = parse_pos(",");
        assert!(res.is_err());
        let res = parse_pos("1,");
        assert!(res.is_err());
        let res = parse_pos("1-");
        assert!(res.is_err());
        let res = parse_pos("1-1-1");
        assert!(res.is_err());
        let res = parse_pos("1-1-a");
        assert!(res.is_err());
        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
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
}

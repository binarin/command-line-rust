// Things to do:
// • Read and write a delimited text file using the csv crate
// • Deference a value using *
// • Use Iterator::flatten to remove nested structures from iterators
// • Use Iterator::flat_map to combine Iterator::map and Iterator::flatten

use std::ops::Range;

use anyhow::Result;
use anyhow::bail;
use anyhow::Error;
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
    #[arg(short, long)]
    fields: Option<String>,

    /// Selected bytes
    #[arg(short, long)]
    bytes: Option<String>,

    /// Selected chars
    #[arg(short, long)]
    chars: Option<String>,
}

type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

fn parse_single_position(s: &str) -> Result<usize> {
    s.parse().map_err(Error::new).and_then(|val: usize| {
        match val {
            v if v > 0 => Ok(v),
            _ => bail!("Failed to parse '{s}'"),
        }
    })
}

fn parse_pos(pos: String) -> Result<PositionList> {
    pos.split(',')
        .map(|range| match range.split_once('-') {
            Some((fst, snd)) => Ok(Range {
                start: parse_single_position(fst)?,
                end: parse_single_position(snd)?,
            }),
            _ => Ok(parse_single_position(range).map(|start| Range {
                start,
                end: start + 1,
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
    dbg!(args);
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
        let pr = parse_pos(s.to_string()).unwrap();
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
        test_parse_pos("5", vec![(5, 6)]);
        test_parse_pos("5,1", vec![(5, 6), (1, 2)]);
    }

    #[test]
    fn parse_pos_range() {
        test_parse_pos("9-15", vec![(9, 15)]);
        test_parse_pos("9-15,14-31,8", vec![(9, 15), (14, 31), (8, 9)]);
    }

    #[test]
    fn test_parse_pos_from_book() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Failed to parse '0'"#);

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Failed to parse '0'"#);

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);
        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );
        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );
        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);
        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);
        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);
        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);
        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());
        let res = parse_pos(",".to_string());
        assert!(res.is_err());
        let res = parse_pos("1,".to_string());
        assert!(res.is_err());
        let res = parse_pos("1-".to_string());
        assert!(res.is_err());
        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());
        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());
        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );
        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );
        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);
        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);
        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);
        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);
        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}

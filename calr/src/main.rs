use anyhow::{Result, anyhow, bail};
use clap::Parser;

/// Rust version of ‘cal’
#[derive(Debug, Parser)]
#[command(about, version, author)]
struct CLIArgs {
    /// Year (1-9999)
    #[arg(value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(short, value_parser = month_arg_parser)]
    month: Option<String>,

    /// Show whole current year
    #[arg(
        short = 'y',
        long = "year",
        default_value_t = false,
        conflicts_with_all = ["month", "year"],
    )]
    show_current_year: bool,
}

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    dbg!(args);
    Ok(())
}

fn month_arg_parser(arg: &str) -> Result<u32> {
    if arg.chars().all(char::is_numeric) {
        let month = arg.parse::<u32>().unwrap();
        if month == 0 || month > 12 {
            return Err(anyhow!(r#"month "{arg}" not in the range 1 through 12"#));
        }
        return Ok(month);
    }
    Ok(match arg.to_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => bail!(r#"Invalid month "{arg}""#),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_month_arg_parser() {
        let res = month_arg_parser("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = month_arg_parser("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);
        let res = month_arg_parser("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = month_arg_parser("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "0" not in the range 1 through 12"#
        );
        let res = month_arg_parser("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"month "13" not in the range 1 through 12"#
        );
        let res = month_arg_parser("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"Invalid month "foo""#);
    }
}

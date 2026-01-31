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

const MONTH_NAMES: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "septemper",
    "october",
    "november",
    "december",
];

fn month_arg_parser(arg: &str) -> Result<u32> {
    if arg.chars().all(char::is_numeric) {
        let month = arg.parse::<u32>().unwrap();
        if (1..=12).contains(&month) {
            return Ok(month);
        }
        return Err(anyhow!(r#"month "{arg}" not in the range 1 through 12"#));
    }

    let candidates: Vec<(u32, &str)> = (1..=12)
        .zip(MONTH_NAMES)
        .filter(|(_, n)| n.starts_with(arg))
        .collect();

    match candidates.as_slice() {
        [(idx, _)] => return Ok(*idx),
        [_, ..] => bail!(r#"Ambigous month name "{arg}""#),
        [] => bail!(r#"Invalid month "{arg}""#),
    }
}

#[cfg(test)]
mod tests {
    use assertables::*;
    use learnr::assert_err_str_contains;

    use super::*;

    #[test]
    fn test_month_arg_parser() {
        let res = month_arg_parser("1");
        assert_ok_eq_x!(res, 1);

        let res = month_arg_parser("12");
        assert_ok_eq_x!(res, 12);

        let res = month_arg_parser("jan");
        assert_ok_eq_x!(res, 1);

        let res = month_arg_parser("0");
        assert_err_str_contains!(res, r#"month "0" not in the range 1 through 12"#);

        let res = month_arg_parser("13");
        assert_err_str_contains!(res, r#"month "13" not in the range 1 through 12"#);

        let res = month_arg_parser("foo");
        assert_err_str_contains!(res, r#"Invalid month "foo""#);

        assert_err_str_contains!(month_arg_parser("ju"), "Ambigous");
    }
}

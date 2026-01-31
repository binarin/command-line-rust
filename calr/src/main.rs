use anyhow::{Result, anyhow, bail};
use chrono::{Datelike, NaiveDate};
use clap::Parser;
use itertools::{Itertools, cons_tuples};

/// Rust version of ‘cal’
#[derive(Debug, Parser)]
#[command(about, version, author)]
struct CLIArgs {
    /// Year (1-9999)
    #[arg(value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(short, value_parser = month_arg_parser)]
    month: Option<u32>,

    /// Show whole current year
    #[arg(
        short = 'y',
        long = "year",
        default_value_t = false,
        conflicts_with_all = ["month", "year"],
    )]
    show_current_year: bool,
}

#[derive(Debug)]
enum Period {
    Month(i32, u32),
    Year(i32),
}

#[derive(Debug)]
struct Args {
    period: Period,
}

fn main() -> Result<()> {
    let args = parse_args(&CLIArgs::parse())?;
    let today = chrono::Local::now().date_naive();

    match args.period {
        Period::Month(year, month) => {
            format_month(year, month, true, today)
                .into_iter()
                .for_each(|l| println!("{}", l));
        }
        Period::Year(year) => {
            for (idx, block_lines) in (1..=12)
                .map(|month| format_month(year, month, false, today))
                .chunks(3)
                .into_iter()
                .map(
                    |triplet| match triplet.collect::<Vec<Vec<String>>>().as_slice() {
                        [m1, m2, m3] => cons_tuples(m1.iter().zip(m2).zip(m3))
                            .map(|(l1, l2, l3)| format!("{l1}{l2}{l3}"))
                            .collect::<Vec<String>>(),
                        _ => {
                            panic!("strange month chunk")
                        }
                    },
                )
                .enumerate()
            {
                if idx == 0 {
                    println!("{year:>width$}", width = BLOCK_WIDTH * 3 / 2 + 2);
                } else {
                    println!();
                };
                block_lines.iter().for_each(|l| println!("{l}"));
            }
        }
    }
    Ok(())
}

fn parse_args(cli_args: &CLIArgs) -> Result<Args> {
    let now = chrono::Local::now();
    let period = match (cli_args.year, cli_args.month, cli_args.show_current_year) {
        (_, _, true) => Period::Year(now.year()),
        (None, None, _) => Period::Month(now.year(), now.month()),
        (Some(year), None, _) => Period::Year(year),
        (None, Some(month), false) => Period::Month(now.year(), month),
        (Some(year), Some(month), false) => Period::Month(year, month),
    };

    Ok(Args { period })
}

const BLOCK_WIDTH: usize = 2 /* sun */ + 3 * 6 /* mon-sat */;
const HORIZONTAL_SEPARATOR: &str = "  ";

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let mut label: String = MONTH_NAMES[month as usize - 1].to_string();
    if print_year {
        label += &format!(" {year}").to_string();
    }
    let mut rows = vec![
        format!("{label:^width$}", width = BLOCK_WIDTH,),
        "Su Mo Tu We Th Fr Sa".to_string(),
    ];

    let today_day: u32 = if year == today.year() && month == today.month() {
        today.day()
    } else {
        u32::MAX
    };

    let dt = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    let mut days: Vec<String> = vec![];

    let filler_num = dt.weekday().number_from_sunday() - 1;
    (1..=filler_num).for_each(|_| days.push("  ".to_string()));
    (1..=dt.num_days_in_month()).for_each(|day| {
        let mut rendered = format!("{day:>2}");
        if today_day == day.into() {
            rendered = ansi_term::Style::new()
                .reverse()
                .paint(rendered)
                .to_string();
        }
        days.push(rendered);
    });
    (days.len()..42).for_each(|_| days.push("  ".to_string()));

    rows.extend(
        days.into_iter()
            .chunks(7)
            .into_iter()
            .map(|ds| itertools::join(ds, " ")),
    );

    rows.iter_mut()
        .for_each(|r: &mut String| *r += HORIZONTAL_SEPARATOR);
    rows
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn month_arg_parser(arg: &str) -> Result<u32> {
    if arg.chars().all(char::is_numeric) {
        let month = arg.parse::<u32>().unwrap();
        if (1..=12).contains(&month) {
            return Ok(month);
        }
        return Err(anyhow!(r#"month "{arg}" not in the range 1 through 12"#));
    }

    let candidates: Vec<(String, u32)> = MONTH_NAMES
        .into_iter()
        .map(str::to_lowercase)
        .zip(1..=12)
        .filter(|(n, _)| n.starts_with(arg))
        .collect();

    match candidates.as_slice() {
        [(_, idx)] => return Ok(*idx),
        [_, ..] => bail!(r#"Ambigous month name "{arg}""#),
        [] => bail!(r#"Invalid month "{arg}""#),
    }
}

#[cfg(test)]
mod tests {
    use assertables::*;
    use learnr::assert_err_str_contains;
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
}

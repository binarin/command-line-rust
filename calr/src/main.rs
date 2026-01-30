use anyhow::{Result, anyhow};
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
    unimplemented!()
}

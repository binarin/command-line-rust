use anyhow::{Result, bail};
use clap::Parser;
use learnr::{CLIInput, open};
use std::{cmp::Ordering, io::BufRead};

/// ’comm’ in Rust
#[derive(Debug, Parser)]
#[command(about, version, author)]
pub struct Args {
    #[arg(value_name = "FILE1")]
    file1: CLIInput,

    #[arg(value_name = "FILE2")]
    file2: CLIInput,

    /// suppress column 1 (lines unique to FILE1)
    #[arg(short('1'), action=clap::ArgAction::SetFalse)]
    show_col1: bool,

    /// suppress column 2 (lines unique to FILE2)
    #[arg(short('2'), action=clap::ArgAction::SetFalse)]
    show_col2: bool,

    /// suppress column 3 (lines that appear in both files)
    #[arg(short('3'), action=clap::ArgAction::SetFalse)]
    show_col3: bool,

    /// compare ignoring case
    #[arg(short('i'))]
    insensitive: bool,

    /// separate columns with STR
    #[arg(
        short('d'),
        long("output-delimiter"),
        default_value = "\t",
        value_name = "STR"
    )]
    delimiter: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.file1 == CLIInput::StdIn && args.file2 == CLIInput::StdIn {
        bail!(r#"Both input files cannot be STDIN ("-")"#);
    }
    let fh1 = open(&args.file1)?;
    let fh2 = open(&args.file2)?;

    let mut iter1 = fh1.lines();
    let mut iter2 = fh2.lines();

    let mut line1 = iter1.next().transpose()?;
    let mut line2 = iter2.next().transpose()?;

    let c2_prefix = if args.show_col1 {
        args.delimiter.clone()
    } else {
        String::new()
    };
    let c3_prefix = if args.show_col2 {
        c2_prefix.clone() + &args.delimiter
    } else {
        c2_prefix.clone()
    };

    loop {
        let ord = match (&line1, &line2) {
            (None, None) => break,
            (Some(s1), Some(s2)) => {
                if args.insensitive {
                    s1.to_lowercase().cmp(&s2.to_lowercase())
                } else {
                    s1.cmp(s2)
                }
            }

            // EOF is always the biggest
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
        };

        // l1 ? l2
        match ord {
            Ordering::Less => {
                if args.show_col1 {
                    println!("{}", line1.unwrap());
                }
                line1 = iter1.next().transpose()?;
            }
            Ordering::Greater => {
                if args.show_col2 {
                    println!("{c2_prefix}{}", line2.unwrap());
                }
                line2 = iter2.next().transpose()?;
            }
            Ordering::Equal => {
                if args.show_col3 {
                    println!("{c3_prefix}{}", line1.unwrap());
                }
                line1 = iter1.next().transpose()?;
                line2 = iter2.next().transpose()?;
            }
        }
    }

    Ok(())
}

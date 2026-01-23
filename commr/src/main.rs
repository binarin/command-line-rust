use anyhow::{Result, bail};
use clap::Parser;
use learnr::{CLIInput, open};

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
    let _fh1 = open(&args.file1)?;
    let _fh2 = open(&args.file2)?;
    dbg!(args);
    Ok(())
}

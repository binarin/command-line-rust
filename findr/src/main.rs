use anyhow::Result;
use clap::{Parser, ValueEnum, builder::PossibleValue};
use regex::Regex;

/// ‘find’ implementation in Rust
#[derive(Debug, Parser)]
#[command(version, about, author)]
struct Args {
    /// Starting points for search
    #[arg(default_value = ".", value_name = "starting-point")]
    paths: Vec<String>,

    /// Expressions
    #[arg(default_value = ".", value_name = "expression", long("name"), short('n'), num_args(0..))]
    names: Vec<Regex>,

    /// File types
    #[arg(long("type"), short('t'), value_name("TYPE"), num_args(0..))]
    entry_types: Vec<EntryType>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[EntryType::Dir, EntryType::File, EntryType::Link]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            EntryType::Dir => PossibleValue::new("d"),
            EntryType::File => PossibleValue::new("f"),
            EntryType::Link => PossibleValue::new("l"),
        })
    }
}

fn main() -> Result<()> {
    dbg!(Args::parse());
    Ok(())
}

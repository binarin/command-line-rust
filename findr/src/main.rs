use std::{fs::Metadata, path::Path};

use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum, builder::PossibleValue};
use regex::Regex;
use walkdir::WalkDir;

/// ‘find’ implementation in Rust
#[derive(Debug, Parser)]
#[command(version, about, author)]
struct Args {
    /// Starting points for search
    #[arg(default_value = ".", value_name = "starting-point")]
    paths: Vec<String>,

    /// Expressions
    #[arg(value_name = "expression", long("name"), short('n'), num_args(0..))]
    names: Option<Vec<Regex>>,

    /// File types
    #[arg(long("type"), short('t'), value_name("TYPE"), num_args(0..))]
    entry_types: Option<Vec<EntryType>>,
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
    let args = Args::parse();
    for path in args.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => {
                    let metadata = entry.metadata()?;
                    let path = entry_filename(&entry)?;

                    if select_type(&metadata, &args.entry_types) && select_name(path, &args.names) {
                        println!("{}", entry.path().display());
                    }
                }
                Err(err) => eprint!("{err}"),
            }
        }
    }
    Ok(())
}

fn select_name(path: &str, regexes: &Option<Vec<Regex>>) -> bool {
    match regexes {
        None => return true,
        Some(regexes) => {
            for re in regexes {
                if re.is_match(path) {
                    return true;
                }
            }
        }
    }
    false
}

fn select_type(metadata: &Metadata, types: &Option<Vec<EntryType>>) -> bool {
    match types {
        None => return true,
        Some(types) => {
            for t in types {
                match t {
                    EntryType::Dir if metadata.is_dir() => return true,
                    EntryType::Link if  metadata.is_symlink() => return true,
                    EntryType::File if metadata.is_file() => return true,
                    _ => (),
                }
            }
        }
    }
    false
}

fn entry_filename(entry: &walkdir::DirEntry) -> Result<&str> {
    let path = entry.path();
    if path == Path::new(".") {
        return Ok(".");
    } else if path == Path::new("..") {
        return Ok("..");
    }
    match path.file_name() {
        None => Err(anyhow!("Failed to get file_name from {path:?}")),
        Some(file_name) => match file_name.to_str() {
            None => Err(anyhow!("Failed to convert file_name {file_name:?} to str")),
            Some(s) => Ok(s),
        },
    }
}

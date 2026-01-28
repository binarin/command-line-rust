use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use regex::{Regex, RegexBuilder};

/// Rust version of ‘fortune’
#[derive(Debug, Parser)]
#[command[about, author, version]]
struct CLIArgs {
    /// Input files or directories
    #[arg(value_name = "FILE", required = true)]
    sources: Vec<String>,

    /// Pattern
    #[arg(short = 'm', long)]
    pattern: Option<String>,

    /// Case-insensitive pattern matching
    #[arg(short, long)]
    insensitive: bool,

    /// Random seed
    #[arg(short, long)]
    seed: Option<u64>,
}

#[derive(Debug)]
struct Args {
    sources: Vec<String>,
    pattern: Option<Regex>,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    dbg!(args);
    Ok(())
}

fn parse_args() -> Result<Args> {
    let CLIArgs {
        sources,
        pattern,
        insensitive,
        ..
    } = CLIArgs::parse();

    let pattern = pattern
        .map(|pat| {
            RegexBuilder::new(pat.as_str())
                .case_insensitive(insensitive)
                .build()
        })
        .transpose()?;

    Ok(Args { sources, pattern })
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::find_files;
    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );
        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());
        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));
        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }
}

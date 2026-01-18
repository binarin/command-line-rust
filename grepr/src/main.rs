use std::fmt::Display;

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug, Clone, PartialEq)]
enum Input {
    File(String),
    StdIn,
}

/// Rust version of ‘grep’
#[derive(Debug, Parser)]
struct Args {
    /// Search pattern
    #[arg(required = true)]
    pattern: String,

    /// Input files(s)
    #[arg(default_value = "-", value_name = "FILE", value_parser = parse_input)]
    files: Vec<Input>,

    /// Case-insensitive
    #[arg(short, long)]
    insensitive: bool,

    /// Recursive search
    #[arg(short, long)]
    recursive: bool,

    /// Count occurences
    #[arg(short, long)]
    count: bool,

    /// Invert match
    #[arg(short('v'), long("invert-match"))]
    invert: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pattern = regex::RegexBuilder::new(&args.pattern)
        .case_insensitive(args.insensitive)
        .build()
        .map_err(|_e| anyhow!(r#"Invalid pattern "{}""#, args.pattern))?;
    println!(r#"Pattern "{pattern}""#);
    let entries = find_files(&args.files, args.recursive);
    for entry in entries {
        match entry {
            Err(e) => eprintln!("{e}"),
            Ok(file) => println!(r#"file "{file}""#),
        }
    }
    Ok(())
}

fn parse_input(filename: &str) -> Result<Input> {
    match filename {
        "-" => Ok(Input::StdIn),
        _ => Ok(Input::File(filename.to_string())),
    }
}

fn find_files(paths: &[Input], recursive: bool) -> Vec<Result<Input>> {
    let mut result: Vec<Result<Input>> = Vec::new();

    for input in paths {
        let Input::File(path) = input else {
            result.push(Ok(input.clone()));
            continue;
        };

        if !recursive {
            let single_res = std::fs::metadata(path)
                .map_err(From::from)
                .and_then(|metadata| {
                    if metadata.is_dir() {
                        Err(anyhow!("{path} is a directory"))
                    } else {
                        Ok(input.clone())
                    }
                });
            result.push(single_res);
            continue;
        }

        let walk = walkdir::WalkDir::new(path);
        for res in walk {
            match res {
                Err(err) => result.push(Err(From::from(err))),
                Ok(dent) => {
                    if dent.file_type().is_file() {
                        match dent.path().to_str() {
                            None => result.push(Err(anyhow!(
                                "Failed to convert dent path '{dent:?}' to string"
                            ))),
                            Some(s) => result.push(Ok(Input::File(s.to_string()))),
                        }
                    }
                }
            }
        }
    }

    result
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::StdIn => f.write_str("<STDIN>"),
            Input::File(file) => file.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    #[test]
    fn test_find_files() {
        // "-" is a special case, we shouldn’t check whether it exists or not
        let files = find_files(&[Input::StdIn], false);
        assert_eq!(files.len(), 1);
        assert_eq!(*files[0].as_ref().unwrap(), Input::StdIn);

        // Verify that the function finds a file known to exist
        let files = find_files(&[Input::File("./tests/inputs/fox.txt".to_string())], false);
        assert_eq!(files.len(), 1);
        assert_eq!(
            *files[0].as_ref().unwrap(),
            Input::File("./tests/inputs/fox.txt".to_string())
        );

        // The function should reject a directory without the recursive option
        let files = find_files(&[Input::File("./tests/inputs".to_string())], false);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].as_ref().unwrap_err().to_string(),
            "./tests/inputs is a directory"
        );

        // Verify the function recurses to find four files in the directory
        let res = find_files(&[Input::File("./tests/inputs".to_string())], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| {
                let Ok(Input::File(f)) = r else {
                    panic!("No {r:?} expected");
                };
                f
            })
            .map(|r| r.replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[Input::File(bad)], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}

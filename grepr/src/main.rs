use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Result, anyhow};
use clap::Parser;
use regex::Regex;

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
    pattern: String, // XXX make Regex

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

    let entries = find_files(&args.files, args.recursive);
    let show_filenames = entries.len() > 1;

    for entry in entries {
        let do_file = |entry| -> Result<()> {
            let input = entry?;
            let prefix = if show_filenames {
                format!("{input}:")
            } else {
                String::new()
            };
            let fh = open(&input)?;
            let filtered = find_lines(fh, &pattern, args.invert)?;
            if args.count {
                println!("{prefix}{}", filtered.len());
            } else {
                filtered.iter().for_each(|l| print!("{prefix}{l}"));
            }
            Ok(())
        };
        let _ = do_file(entry).map_err(|e| eprintln!("{e:?}"));
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
                .map_err(|err| anyhow!("{path}: {err}"))
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
            Input::StdIn => f.write_str("-"),
            Input::File(file) => file.fmt(f),
        }
    }
}

fn open(input: &Input) -> Result<Box<dyn BufRead>> {
    match input {
        Input::StdIn => Ok(Box::new(BufReader::new(std::io::stdin()))),
        Input::File(file) => Ok(Box::new(BufReader::new(File::open(file)?))),
    }
}

fn find_lines<T: BufRead>(mut file: T, pattern: &Regex, invert: bool) -> Result<Vec<String>> {
    let mut result = vec![];
    loop {
        let mut s = String::new();
        let bytes_read = file.read_line(&mut s)?;
        if bytes_read == 0 {
            break;
        }
        if pattern.is_match(&s) ^ invert {
            result.push(s);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use rand::{Rng, distributions::Alphanumeric};
    use regex::RegexBuilder;
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

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}

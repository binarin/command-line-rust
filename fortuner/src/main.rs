use std::{
    fs::File,
    io::{BufRead, BufReader},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use rand::{Rng, SeedableRng, rngs::StdRng};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

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
    sources: Vec<PathBuf>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let fortunes = read_fortunes(&args.sources)?;
    match &args.pattern {
        None => {
            if fortunes.is_empty() {
                println!("No fortunes found");
                return Ok(());
            }
            let fortune = pick_fortune(&fortunes, args.seed).unwrap();
            println!("{}", fortune);
        }
        Some(pattern) => {
            let mut prev_source: Option<String> = None;
            for Fortune { text, source } in fortunes {
                if pattern.is_match(&text) {
                    if prev_source != Some(source.clone()) {
                        eprintln!("({source})\n%");
                        prev_source = Some(source.clone());
                    }
                    println!("{}\n%", text);
                }
            }
        }
    }
    Ok(())
}

fn parse_args() -> Result<Args> {
    let CLIArgs {
        sources,
        pattern,
        insensitive,
        seed,
    } = CLIArgs::parse();

    let pattern = pattern
        .map(|pat| {
            RegexBuilder::new(pat.as_str())
                .case_insensitive(insensitive)
                .build()
        })
        .transpose()?;

    let sources = find_files(&sources)?;

    Ok(Args {
        sources,
        pattern,
        seed,
    })
}

fn find_single_source(path: &String) -> Result<Vec<PathBuf>> {
    let mut result = vec![];
    for file in WalkDir::new(path).sort_by_file_name() {
        let file = file?;
        if !file.file_type().is_file() {
            continue;
        }
        if file.metadata()?.len() == 0 {
            continue;
        }
        let path = file.into_path();

        if let Some(ext) = path.extension()
            && ext == "dat"
        {
            continue;
        }

        if let Some(name) = path.file_name() {
            if name.len() > 0 && name.as_bytes()[0] == b'.' {
                continue;
            }
        }

        result.push(path);
    }
    Ok(result)
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let mut result = vec![];
    for path in paths {
        result.append(&mut find_single_source(path)?);
    }
    result.sort();
    result.dedup();
    Ok(result)
}

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    let mut result = vec![];

    for path in paths {
        let mut reader = BufReader::new(File::open(path)?);
        loop {
            let mut buf: Vec<u8> = vec![];
            let bytes_read = reader.read_until(b'%', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let text = String::from_utf8_lossy(&buf)
                .trim_matches(&['%', '\n'])
                .to_string();
            if text.is_empty() {
                continue;
            }
            result.push(Fortune {
                source: path
                    .file_name()
                    .expect("source should have filename")
                    .to_string_lossy()
                    .into_owned(),
                text,
            });
        }
    }

    Ok(result)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if fortunes.is_empty() {
        return None;
    }
    let mut rng = match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_rng(rand::thread_rng()).expect("seeding from thread_rnd"),
    };
    let pick = rng.gen_range(0..fortunes.len());
    Some(fortunes[pick].text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(files.len(), 4);
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

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());
        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
A: A bad idea (bad-eye deer)."
            );
        }
        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];
        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}

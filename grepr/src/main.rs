use anyhow::{Result, anyhow};
use clap::Parser;

/// Rust version of ‘grep’
#[derive(Debug, Parser)]
struct Args {
    /// Search pattern
    #[arg(required = true)]
    pattern: String,

    /// Input files(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

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
        .map_err(|e| anyhow!(r#"Invalid pattern "{}""#, args.pattern))?;
    println!(r#"Pattern "{pattern}""#);
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<Result<String>> {
    let mut result = Vec::new();

    for path in paths {
        if !recursive {
            let single_res = std::fs::metadata(path)
                .map_err(From::from)
                .and_then(|metadata| {
                    if metadata.is_dir() {
                        Err(anyhow!("'{path}' is a directory"))
                    } else {
                        Ok(path.to_string())
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
                            Some(s) => result.push(Ok(s.to_string())),
                        }
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{Rng, distributions::Alphanumeric};
    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }
        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
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
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}

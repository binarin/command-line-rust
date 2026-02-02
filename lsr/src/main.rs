use std::{
    fs::{DirEntry, metadata, read_dir},
    io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use tabular::{Row, Table};

/// Rust version of ’ls’
#[derive(Debug, Parser)]
#[command(author, about, version)]
struct CLIArgs {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,

    /// Long listing
    #[arg(short, long)]
    long: bool,

    /// Show all files
    #[arg(short = 'a', long = "all")]
    show_hidden: bool,
}

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let paths = find_files(&args.paths, args.show_hidden)?;
    if args.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn find_files(paths: &[PathBuf], show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut result = vec![];

    for path in paths {
        let process_dir_entry = |rde: Result<DirEntry, io::Error>| -> Option<PathBuf> {
            rde.map_or(None, |de| {
                if de.file_name().as_encoded_bytes().starts_with(b".") && !show_hidden {
                    return None;
                }
                Some(de.path())
            })
        };

        match metadata(path) {
            Ok(meta) => {
                if meta.file_type().is_dir() {
                    match read_dir(path) {
                        Ok(entries) => result.extend(entries.filter_map(process_dir_entry)),
                        Err(e) => eprintln!("ls: {}: {e}", path.display()),
                    }
                } else {
                    result.push(path.to_path_buf());
                }
            }
            Err(e) => eprintln!("ls: {}: {e}", path.display()),
        }
    }

    Ok(result)
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);
    for path in paths {
        let metadata = match metadata(path) {
            Ok(md) => md,
            Err(err) => {
                eprintln!("{path}: {err}", path = path.display());
                continue;
            }
        };

        let uid = metadata.uid();
        let username = users::get_user_by_uid(uid)
            .map(|name| name.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());

        let gid = metadata.uid();
        let group: String = if let Some(name) = users::get_group_by_gid(gid) {
            name.name().to_string_lossy().to_string()
        } else {
            format!("{gid}")
        };

        let modified: String = match metadata.modified() {
            Ok(modified) => chrono::DateTime::<Local>::from(modified)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            Err(err) => {
                eprintln!("{}: {err}", path.display());
                continue;
            }
        };

        table.add_row(
            Row::new()
                .with_cell(if metadata.is_dir() { "d" } else { "-" })
                .with_cell(format_permissions(&metadata))
                .with_cell(metadata.nlink())
                .with_cell(username)
                .with_cell(group)
                .with_cell(metadata.size())
                .with_cell(modified)
                .with_cell(path.display()),
        );
    }
    Ok(format!("{table}"))
}

fn format_permissions(metadata: &std::fs::Metadata) -> String {
    let mut bits: Vec<bool> = vec![];
    let mut mode = metadata.permissions().mode();
    while bits.len() < 9 {
        bits.push(mode % 2 == 1);
        mode /= 2;
    }

    let bit_strs = ['x', 'w', 'r'].into_iter().cycle().take(9);
    let permission_str: String = bits
        .iter()
        .zip(bit_strs)
        .map(|(bit, repr)| if *bit { repr } else { '-' })
        .collect();

    let permission_str: String = permission_str.chars().rev().collect();
    permission_str
}

// --------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! assert_find_files {
        ($expected:expr, $show_hidden:expr, $($path:expr),+ $(,)?) => {{
            let res = find_files(&[$($path.into()),+], $show_hidden);
            assert!(res.is_ok());
            let mut filenames: Vec<_> = res
                .unwrap()
                .iter()
                .map(|entry| entry.display().to_string())
                .collect();
            filenames.sort();
            let mut expected = $expected.to_vec();
            expected.sort();
            assert_eq!(filenames, expected);
        }};
    }

    #[test]
    fn test_find_files_nonhidden() {
        assert_find_files!(
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ],
            false,
            "tests/inputs"
        );
    }

    #[test]
    fn test_find_files_hidden_explicit() {
        assert_find_files!(["tests/inputs/.hidden"], false, "tests/inputs/.hidden");
    }

    #[test]
    fn test_find_files_multiple_paths() {
        assert_find_files!(
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"],
            false,
            "tests/inputs/bustle.txt",
            "tests/inputs/dir"
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        assert_find_files!(
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ],
            true,
            "tests/inputs",
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    // #[test]
    // fn test_mk_triple() {
    //     assert_eq!(mk_triple(0o751, Owner::User), "rwx");
    //     assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
    //     assert_eq!(mk_triple(0o751, Owner::Other), "--x");
    //     assert_eq!(mk_triple(0o600, Owner::Other), "---");
    // }

    // #[test]
    // fn test_format_mode() {
    //     assert_eq!(format_mode(0o755), "rwxr-xr-x");
    //     assert_eq!(format_mode(0o421), "r---w---x");
    // }
}

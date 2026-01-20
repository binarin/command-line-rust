use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use rand::{Rng, distributions::Alphanumeric};
use std::{fs, path::Path};
use sys_info::os_type;

const BUSTLE: &str = "tests/inputs/bustle.txt";
const EMPTY: &str = "tests/inputs/empty.txt";
const FOX: &str = "tests/inputs/fox.txt";
const NOBODY: &str = "tests/inputs/nobody.txt";
const INPUTS_DIR: &str = "tests/inputs";

// --------------------------------------------------
fn gen_bad_file() -> String {
    loop {
        let filename: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}

// --------------------------------------------------
#[test]
fn dies_no_args() -> Result<()> {
    cargo_bin_cmd!()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_pattern() -> Result<()> {
    cargo_bin_cmd!()
        .args(["*foo", FOX])
        .assert()
        .failure()
        .stderr(predicate::str::contains(r#"Invalid pattern "*foo""#));
    Ok(())
}

// --------------------------------------------------
#[test]
fn warns_bad_file() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    cargo_bin_cmd!()
        .args(["foo", &bad])
        .assert()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

macro_rules! run {
    ($expected_file:expr, $($args:expr),* $(,)? ) => {
        {
            let args = [$($args),*];
            let expected_file: String = From::from($expected_file);
            let windows_file: String = format!("{expected_file}.windows");
            let expected_file = if os_type().unwrap() == "Windows" && Path::new(&windows_file).is_file() {
                windows_file
            } else {
                expected_file
            };

            let expected = fs::read_to_string(expected_file).expect("input-fail");
            let output = cargo_bin_cmd!().args(args).output().expect("fail");
            assert!(output.status.success());

            let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
            assert_eq!(stdout, expected);
            Ok(())
        }
    };
}

// --------------------------------------------------
#[test]
fn empty_file() -> Result<()> {
    run!("tests/expected/empty.foo", "foo", EMPTY)
}

// --------------------------------------------------
#[test]
fn empty_regex() -> Result<()> {
    run!("tests/expected/empty_regex.fox.txt", "", FOX)
}

// --------------------------------------------------
#[test]
fn bustle_capitalized() -> Result<()> {
    run!("tests/expected/bustle.txt.the.capitalized", "The", BUSTLE,)
}

// --------------------------------------------------
#[test]
fn bustle_lowercase() -> Result<()> {
    run!("tests/expected/bustle.txt.the.lowercase", "the", BUSTLE)
}

// --------------------------------------------------
#[test]
fn bustle_insensitive() -> Result<()> {
    run!(
        "tests/expected/bustle.txt.the.lowercase.insensitive",
        "--insensitive",
        "the",
        BUSTLE
    )
}

// --------------------------------------------------
#[test]
fn nobody() -> Result<()> {
    run!("tests/expected/nobody.txt", "nobody", NOBODY)
}

// --------------------------------------------------
#[test]
fn nobody_insensitive() -> Result<()> {
    run!(
        "tests/expected/nobody.txt.insensitive",
        "-i",
        "nobody",
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn multiple_files() -> Result<()> {
    run!(
        "tests/expected/all.the.capitalized",
        "The",
        BUSTLE,
        EMPTY,
        FOX,
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn multiple_files_insensitive() -> Result<()> {
    run!(
        "tests/expected/all.the.lowercase.insensitive",
        "-i",
        "the",
        BUSTLE,
        EMPTY,
        FOX,
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn recursive() -> Result<()> {
    run!(
        "tests/expected/dog.recursive",
        "--recursive",
        "dog",
        INPUTS_DIR,
    )
}

// --------------------------------------------------
#[test]
fn recursive_insensitive() -> Result<()> {
    run!(
        "tests/expected/the.recursive.insensitive",
        "-ri",
        "then",
        INPUTS_DIR,
    )
}

// --------------------------------------------------
#[test]
fn sensitive_count_capital() -> Result<()> {
    run!(
        "tests/expected/bustle.txt.the.capitalized.count",
        "--count",
        "The",
        BUSTLE,
    )
}

// --------------------------------------------------
#[test]
fn sensitive_count_lower() -> Result<()> {
    run!(
        "tests/expected/bustle.txt.the.lowercase.count",
        "--count",
        "the",
        BUSTLE,
    )
}

// --------------------------------------------------
#[test]
fn insensitive_count() -> Result<()> {
    run!(
        "tests/expected/bustle.txt.the.lowercase.insensitive.count",
        "-ci",
        "the",
        BUSTLE,
    )
}

// --------------------------------------------------
#[test]
fn nobody_count() -> Result<()> {
    run!("tests/expected/nobody.txt.count", "-c", "nobody", NOBODY)
}

// --------------------------------------------------
#[test]
fn nobody_count_insensitive() -> Result<()> {
    run!(
        "tests/expected/nobody.txt.insensitive.count",
        "-ci",
        "nobody",
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn sensitive_count_multiple() -> Result<()> {
    run!(
        "tests/expected/all.the.capitalized.count",
        "-c",
        "The",
        BUSTLE,
        EMPTY,
        FOX,
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn insensitive_count_multiple() -> Result<()> {
    run!(
        "tests/expected/all.the.lowercase.insensitive.count",
        "-ic",
        "the",
        BUSTLE,
        EMPTY,
        FOX,
        NOBODY,
    )
}

// --------------------------------------------------
#[test]
fn warns_dir_not_recursive() -> Result<()> {
    let stdout = "tests/inputs/fox.txt:\
        The quick brown fox jumps over the lazy dog.";
    cargo_bin_cmd!()
        .args(["fox", INPUTS_DIR, FOX])
        .assert()
        .stderr(predicate::str::contains("tests/inputs is a directory"))
        .stdout(predicate::str::contains(stdout));
    Ok(())
}

// --------------------------------------------------
#[test]
fn stdin() -> Result<()> {
    let input = fs::read_to_string(BUSTLE)?;
    let expected = fs::read_to_string("tests/expected/bustle.txt.the.capitalized")?;

    let output = cargo_bin_cmd!()
        .arg("The")
        .write_stdin(input)
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

// --------------------------------------------------
#[test]
fn stdin_insensitive_count() -> Result<()> {
    let files = &[BUSTLE, EMPTY, FOX, NOBODY];

    let mut input = String::new();
    for file in files {
        input += &fs::read_to_string(file)?;
    }

    let expected_file = "tests/expected/the.recursive.insensitive.count.stdin";
    let expected = fs::read_to_string(expected_file)?;

    let output = cargo_bin_cmd!()
        .args(["-ci", "the", "-"])
        .write_stdin(input)
        .output()
        .expect("fail");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
    Ok(())
}

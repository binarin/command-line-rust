use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use rand::{Rng, distributions::Alphanumeric};
use std::fs;

const EMPTY: &str = "tests/inputs/empty.txt";
const FILE1: &str = "tests/inputs/file1.txt";
const FILE2: &str = "tests/inputs/file2.txt";
const BLANK: &str = "tests/inputs/blank.txt";

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
fn dies_bad_file1() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    cargo_bin_cmd!()
        .args([&bad, FILE1])
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_file2() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    cargo_bin_cmd!()
        .args([FILE1, &bad])
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_both_stdin() -> Result<()> {
    let expected = r#"Both input files cannot be STDIN ("-")"#;
    cargo_bin_cmd!()
        .args(["-", "-"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
macro_rules! run {
    ($expected_file:expr , $($args:expr),* $(,)? ) => {{
        let expected_file: String = From::from($expected_file);
        let args = [ $($args),* ];
        let expected = fs::read_to_string(expected_file).expect("infile-fail");
        let output = cargo_bin_cmd!().args(args).output().expect("fail");
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
        assert_eq!(stdout, expected);
        Ok(())
    }};
}

// --------------------------------------------------
macro_rules! run_stdin {
    ($input_file:expr , $expected_file:expr , $($args:expr),* $(,)? ) => {{
        let input_file: String = From::from($input_file);
        let input = fs::read_to_string(input_file.as_str()).expect("input-file");

        let expected_file: String = From::from($expected_file);
        let expected = fs::read_to_string(expected_file.as_str()).expect("expected-file");

        let output = cargo_bin_cmd!()
            .args([ $($args),* ])
            .write_stdin(input)
            .output()
            .expect("fail");
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
        assert_eq!(stdout, expected);
        Ok(())
    }};
}

// --------------------------------------------------
#[test]
fn empty_empty() -> Result<()> {
    run!("tests/expected/empty_empty.out", EMPTY, EMPTY)
}

// --------------------------------------------------
#[test]
fn file1_file1() -> Result<()> {
    run!("tests/expected/file1_file1.out", FILE1, FILE1)
}

// --------------------------------------------------
#[test]
fn file1_file2() -> Result<()> {
    run!("tests/expected/file1_file2.out", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_empty() -> Result<()> {
    run!("tests/expected/file1_empty.out", FILE1, EMPTY)
}

// --------------------------------------------------
#[test]
fn empty_file2() -> Result<()> {
    run!("tests/expected/empty_file2.out", EMPTY, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_1() -> Result<()> {
    run!("tests/expected/file1_file2.1.out", "-1", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_2() -> Result<()> {
    run!("tests/expected/file1_file2.2.out", "-2", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_3() -> Result<()> {
    run!("tests/expected/file1_file2.3.out", "-3", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_1_2() -> Result<()> {
    run!("tests/expected/file1_file2.12.out", "-12", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_2_3() -> Result<()> {
    run!("tests/expected/file1_file2.23.out", "-23", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_13() -> Result<()> {
    run!("tests/expected/file1_file2.13.out", "-13", FILE1, FILE2)
}

// --------------------------------------------------
#[test]
fn file1_file2_123() -> Result<()> {
    run!("tests/expected/file1_file2.123.out", "-123", FILE1, FILE2)
}

// --------------------------------------------------
// insensitive
// --------------------------------------------------
#[test]
fn file1_file2_1_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.1.i.out",
        "-1",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_2_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.2.i.out",
        "-2",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_3_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.3.i.out",
        "-3",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_1_2_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.12.i.out",
        "-12",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_2_3_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.23.i.out",
        "-23",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_13_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.13.i.out",
        "-13",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_123_i() -> Result<()> {
    run!(
        "tests/expected/file1_file2.123.i.out",
        "-123",
        "-i",
        FILE1,
        FILE2
    )
}

// --------------------------------------------------
#[test]
fn stdin_file1() -> Result<()> {
    run_stdin!(
        FILE1,
        "tests/expected/file1_file2.123.i.out",
        "-123",
        "-i",
        "-",
        FILE2,
    )
}

// --------------------------------------------------
#[test]
fn stdin_file2() -> Result<()> {
    run_stdin!(
        FILE2,
        "tests/expected/file1_file2.123.i.out",
        "-123",
        "-i",
        FILE1,
        "-",
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.delim.out",
        FILE1,
        FILE2,
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_1_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.1.delim.out",
        FILE1,
        FILE2,
        "-1",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_2_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.2.delim.out",
        FILE1,
        FILE2,
        "-2",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_3_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.3.delim.out",
        FILE1,
        FILE2,
        "-3",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_12_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.12.delim.out",
        FILE1,
        FILE2,
        "-12",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_23_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.23.delim.out",
        FILE1,
        FILE2,
        "-23",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_13_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.13.delim.out",
        FILE1,
        FILE2,
        "-13",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn file1_file2_123_delim() -> Result<()> {
    run!(
        "tests/expected/file1_file2.123.delim.out",
        FILE1,
        FILE2,
        "-123",
        "-d",
        ":"
    )
}

// --------------------------------------------------
#[test]
fn blank_file1() -> Result<()> {
    run!("tests/expected/blank_file1.out", BLANK, FILE1)
}

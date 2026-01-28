use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use rand::{Rng, distributions::Alphanumeric};
use std::fs;

const FORTUNE_DIR: &str = "./tests/inputs";
const EMPTY_DIR: &str = "./tests/inputs/empty";
const JOKES: &str = "./tests/inputs/jokes";
const LITERATURE: &str = "./tests/inputs/literature";

// --------------------------------------------------
fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

// --------------------------------------------------
fn gen_bad_file() -> String {
    loop {
        let filename = random_string();
        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}

// --------------------------------------------------
macro_rules! run {
    ($expected:expr , $($args:expr),* $(,)? ) => {{
        let expected: &str = $expected;
        let output = cargo_bin_cmd!().args([ $($args),* ]).output().expect("fail");
        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
        assert_eq!(stdout, expected);
        Ok(())
    }};
}

// --------------------------------------------------
macro_rules! run_outfiles {
    ($out_file:expr , $err_file:expr , $($args:expr),* $(,)? ) => {{
        let expected_out = fs::read_to_string($out_file).expect("out-file");
        let expected_err = fs::read_to_string($err_file).expect("err-file");

        let output = cargo_bin_cmd!().args([ $($args),* ]).output().expect("fail");
        assert!(output.status.success());

        let stdout = String::from_utf8(output.clone().stdout).expect("invalid UTF-8");
        assert_eq!(stdout, expected_out);

        let stderr = String::from_utf8(output.stderr).expect("invalid UTF-8");
        assert_eq!(stderr, expected_err);

        Ok(())
    }};
}

// --------------------------------------------------
#[test]
fn dies_not_enough_args() -> Result<()> {
    let expected = "the following required arguments were not provided:\n  \
        <FILE>...";
    cargo_bin_cmd!()
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_file() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    cargo_bin_cmd!()
        .args([LITERATURE, &bad])
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_pattern() -> Result<()> {
    let expected = r#"Error: regex parse error:"#;
    cargo_bin_cmd!()
        .args(["--pattern", "*", LITERATURE])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_seed() -> Result<()> {
    let bad = random_string();
    let expected = format!("invalid value '{bad}' for '--seed <SEED>'");
    cargo_bin_cmd!()
        .args([LITERATURE, "--seed", &bad])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[test]
fn no_fortunes_found() -> Result<()> {
    run!("No fortunes found\n", EMPTY_DIR)
}

// --------------------------------------------------
#[test]
fn quotes_seed_1() -> Result<()> {
    run!(
        "You can observe a lot just by watching.\n-- Yogi Berra\n",
        FORTUNE_DIR,
        "-s",
        "1",
    )
}

// --------------------------------------------------
#[test]
fn jokes_seed_1() -> Result<()> {
    run!(
        "Q: What happens when frogs park illegally?\nA: They get toad.\n",
        JOKES,
        "-s",
        "1",
    )
}

// --------------------------------------------------
#[test]
fn dir_seed_11() -> Result<()> {
    run!(
        "Q: Why did the gardener quit his job?\nA: His celery wasn't high enough.\n",
        FORTUNE_DIR,
        "-s",
        "11",
    )
}

// --------------------------------------------------
#[test]
fn yogi_berra_cap() -> Result<()> {
    run_outfiles!(
        "tests/expected/berra_cap.out",
        "tests/expected/berra_cap.err",
        "--pattern",
        "Yogi Berra",
        FORTUNE_DIR,
    )
}

// --------------------------------------------------
#[test]
fn mark_twain_cap() -> Result<()> {
    run_outfiles!(
        "tests/expected/twain_cap.out",
        "tests/expected/twain_cap.err",
        "-m",
        "Mark Twain",
        FORTUNE_DIR,
    )
}

// --------------------------------------------------
#[test]
fn yogi_berra_lower() -> Result<()> {
    run_outfiles!(
        "tests/expected/berra_lower.out",
        "tests/expected/berra_lower.err",
        "--pattern",
        "yogi berra",
        FORTUNE_DIR,
    )
}

// --------------------------------------------------
#[test]
fn mark_twain_lower() -> Result<()> {
    run_outfiles!(
        "tests/expected/twain_lower.out",
        "tests/expected/twain_lower.err",
        "-m",
        "will twain",
        FORTUNE_DIR,
    )
}

// --------------------------------------------------
#[test]
fn yogi_berra_lower_i() -> Result<()> {
    run_outfiles!(
        "tests/expected/berra_lower_i.out",
        "tests/expected/berra_lower_i.err",
        "--insensitive",
        "--pattern",
        "yogi berra",
        FORTUNE_DIR,
    )
}

// --------------------------------------------------
#[test]
fn mark_twain_lower_i() -> Result<()> {
    run_outfiles!(
        "tests/expected/twain_lower_i.out",
        "tests/expected/twain_lower_i.err",
        "-i",
        "-m",
        "mark twain",
        FORTUNE_DIR,
    )
}

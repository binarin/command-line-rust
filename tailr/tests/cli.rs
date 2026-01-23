use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
use rand::{Rng, distributions::Alphanumeric};
use std::fs::{self, File};
use std::io::Read;
const EMPTY: &str = "tests/inputs/empty.txt";
const ONE: &str = "tests/inputs/one.txt";
const TWO: &str = "tests/inputs/two.txt";
const THREE: &str = "tests/inputs/three.txt";
const TWELVE: &str = "tests/inputs/twelve.txt";

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
fn dies_bad_bytes() -> Result<()> {
    let bad = random_string();
    let expected = format!("illegal byte count -- {bad}");
    cargo_bin_cmd!()
        .args(["-c", &bad, EMPTY])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));

    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bad_lines() -> Result<()> {
    let bad = random_string();
    let expected = format!("illegal line count -- {bad}");
    cargo_bin_cmd!()
        .args(["-n", &bad, EMPTY])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));

    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_bytes_and_lines() -> Result<()> {
    let msg = "the argument '--lines <LINES>' cannot be used \
               with '--bytes <BYTES>'";

    cargo_bin_cmd!()
        .args(["-n", "1", "-c", "2"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(msg));

    Ok(())
}

// --------------------------------------------------
#[test]
fn skips_bad_file() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("{bad}: .* [(]os error 2[)]");
    cargo_bin_cmd!()
        .args([ONE, &bad, TWO])
        .assert()
        .stderr(predicate::str::is_match(expected)?);

    Ok(())
}

// --------------------------------------------------
macro_rules! run {
    ($expected_file:expr , $($args:expr),* $(,)? ) => {{
        let expected_file: String = From::from($expected_file);
        let args = [ $($args),* ];
        // Extra work here due to lossy UTF
        let mut file = File::open(&expected_file).expect("infile-fail");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("read-fail");
        let expected = String::from_utf8_lossy(&buffer);

        let output = cargo_bin_cmd!().args(args).output().expect("fail");
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        Ok(())
    }};
}

// --------------------------------------------------
#[test]
fn empty() -> Result<()> {
    run!("tests/expected/empty.txt.out", EMPTY)
}

#[test]
fn empty_n0() -> Result<()> {
    run!("tests/expected/empty.txt.n0.out", EMPTY, "-n", "0")
}

#[test]
fn empty_n1() -> Result<()> {
    run!("tests/expected/empty.txt.n1.out", EMPTY, "-n", "1")
}

#[test]
fn empty_n_minus_1() -> Result<()> {
    run!("tests/expected/empty.txt.n1.out", EMPTY, "-n=-1")
}

#[test]
fn empty_n3() -> Result<()> {
    run!("tests/expected/empty.txt.n3.out", EMPTY, "-n", "3")
}

#[test]
fn empty_n_minus_3() -> Result<()> {
    run!("tests/expected/empty.txt.n3.out", EMPTY, "-n=-3")
}

#[test]
fn empty_n4() -> Result<()> {
    run!("tests/expected/empty.txt.n4.out", EMPTY, "-n", "4")
}

#[test]
fn empty_n200() -> Result<()> {
    run!("tests/expected/empty.txt.n200.out", EMPTY, "-n", "200")
}

#[test]
fn empty_n_minus_200() -> Result<()> {
    run!("tests/expected/empty.txt.n200.out", EMPTY, "-n=-200")
}

#[test]
fn empty_n_minus_4() -> Result<()> {
    run!("tests/expected/empty.txt.n4.out", EMPTY, "-n=-4")
}

#[test]
fn empty_n_plus_0() -> Result<()> {
    run!("tests/expected/empty.txt.n+0.out", EMPTY, "-n", "+0")
}

#[test]
fn empty_n_plus_1() -> Result<()> {
    run!("tests/expected/empty.txt.n+1.out", EMPTY, "-n", "+1")
}

#[test]
fn empty_n_plus_2() -> Result<()> {
    run!("tests/expected/empty.txt.n+2.out", EMPTY, "-n", "+2")
}

#[test]
fn empty_c3() -> Result<()> {
    run!("tests/expected/empty.txt.c3.out", EMPTY, "-c", "3")
}

#[test]
fn empty_c_minus_3() -> Result<()> {
    run!("tests/expected/empty.txt.c3.out", EMPTY, "-c=-3")
}

#[test]
fn empty_c8() -> Result<()> {
    run!("tests/expected/empty.txt.c8.out", EMPTY, "-c", "8")
}

#[test]
fn empty_c_minus_8() -> Result<()> {
    run!("tests/expected/empty.txt.c8.out", EMPTY, "-c=8")
}

#[test]
fn empty_c12() -> Result<()> {
    run!("tests/expected/empty.txt.c12.out", EMPTY, "-c", "12")
}

#[test]
fn empty_c_minus_12() -> Result<()> {
    run!("tests/expected/empty.txt.c12.out", EMPTY, "-c=-12")
}

#[test]
fn empty_c200() -> Result<()> {
    run!("tests/expected/empty.txt.c200.out", EMPTY, "-c", "200")
}

#[test]
fn empty_c_minus_200() -> Result<()> {
    run!("tests/expected/empty.txt.c200.out", EMPTY, "-c=-200")
}

#[test]
fn empty_c_plus_0() -> Result<()> {
    run!("tests/expected/empty.txt.c+0.out", EMPTY, "-c", "+0")
}

#[test]
fn empty_c_plus_1() -> Result<()> {
    run!("tests/expected/empty.txt.c+1.out", EMPTY, "-c", "+1")
}

#[test]
fn empty_c_plus_2() -> Result<()> {
    run!("tests/expected/empty.txt.c+2.out", EMPTY, "-c", "+2")
}

// --------------------------------------------------
#[test]
fn one() -> Result<()> {
    run!("tests/expected/one.txt.out", ONE)
}

#[test]
fn one_n0() -> Result<()> {
    run!("tests/expected/one.txt.n0.out", ONE, "-n", "0")
}

#[test]
fn one_n1() -> Result<()> {
    run!("tests/expected/one.txt.n1.out", ONE, "-n", "1")
}

#[test]
fn one_n_minus_1() -> Result<()> {
    run!("tests/expected/one.txt.n1.out", ONE, "-n=-1")
}

#[test]
fn one_n3() -> Result<()> {
    run!("tests/expected/one.txt.n3.out", ONE, "-n", "3")
}

#[test]
fn one_n_minus_3() -> Result<()> {
    run!("tests/expected/one.txt.n3.out", ONE, "-n=-3")
}

#[test]
fn one_n4() -> Result<()> {
    run!("tests/expected/one.txt.n4.out", ONE, "-n", "4")
}

#[test]
fn one_n_minus_4() -> Result<()> {
    run!("tests/expected/one.txt.n4.out", ONE, "-n=-4")
}

#[test]
fn one_n200() -> Result<()> {
    run!("tests/expected/one.txt.n200.out", ONE, "-n", "200")
}

#[test]
fn one_n_minus_200() -> Result<()> {
    run!("tests/expected/one.txt.n200.out", ONE, "-n=-200")
}

#[test]
fn one_n_plus_0() -> Result<()> {
    run!("tests/expected/one.txt.n+0.out", ONE, "-n", "+0")
}

#[test]
fn one_n_plus_1() -> Result<()> {
    run!("tests/expected/one.txt.n+1.out", ONE, "-n", "+1")
}

#[test]
fn one_n_plus_2() -> Result<()> {
    run!("tests/expected/one.txt.n+2.out", ONE, "-n", "+2")
}

#[test]
fn one_c3() -> Result<()> {
    run!("tests/expected/one.txt.c3.out", ONE, "-c", "3")
}

#[test]
fn one_c_minus_3() -> Result<()> {
    run!("tests/expected/one.txt.c3.out", ONE, "-c=-3")
}

#[test]
fn one_c8() -> Result<()> {
    run!("tests/expected/one.txt.c8.out", ONE, "-c", "8")
}

#[test]
fn one_c_minus_8() -> Result<()> {
    run!("tests/expected/one.txt.c8.out", ONE, "-c=8")
}

#[test]
fn one_c12() -> Result<()> {
    run!("tests/expected/one.txt.c12.out", ONE, "-c", "12")
}

#[test]
fn one_c_minus_12() -> Result<()> {
    run!("tests/expected/one.txt.c12.out", ONE, "-c=-12")
}

#[test]
fn one_c200() -> Result<()> {
    run!("tests/expected/one.txt.c200.out", ONE, "-c", "200")
}

#[test]
fn one_c_minus_200() -> Result<()> {
    run!("tests/expected/one.txt.c200.out", ONE, "-c=-200")
}

#[test]
fn one_c_plus_0() -> Result<()> {
    run!("tests/expected/one.txt.c+0.out", ONE, "-c", "+0")
}

#[test]
fn one_c_plus_1() -> Result<()> {
    run!("tests/expected/one.txt.c+1.out", ONE, "-c", "+1")
}

#[test]
fn one_c_plus_2() -> Result<()> {
    run!("tests/expected/one.txt.c+2.out", ONE, "-c", "+2")
}

// --------------------------------------------------
#[test]
fn two() -> Result<()> {
    run!("tests/expected/two.txt.out", TWO)
}

#[test]
fn two_n0() -> Result<()> {
    run!("tests/expected/two.txt.n0.out", TWO, "-n", "0")
}

#[test]
fn two_n1() -> Result<()> {
    run!("tests/expected/two.txt.n1.out", TWO, "-n", "1")
}

#[test]
fn two_n_minus_1() -> Result<()> {
    run!("tests/expected/two.txt.n1.out", TWO, "-n=-1")
}

#[test]
fn two_n3() -> Result<()> {
    run!("tests/expected/two.txt.n3.out", TWO, "-n", "3")
}

#[test]
fn two_n_minus_3() -> Result<()> {
    run!("tests/expected/two.txt.n3.out", TWO, "-n=-3")
}

#[test]
fn two_n4() -> Result<()> {
    run!("tests/expected/two.txt.n4.out", TWO, "-n", "4")
}

#[test]
fn two_n_minus_4() -> Result<()> {
    run!("tests/expected/two.txt.n4.out", TWO, "-n=-4")
}

#[test]
fn two_n200() -> Result<()> {
    run!("tests/expected/two.txt.n200.out", TWO, "-n", "200")
}

#[test]
fn two_n_minus_200() -> Result<()> {
    run!("tests/expected/two.txt.n200.out", TWO, "-n=-200")
}

#[test]
fn two_n_plus_0() -> Result<()> {
    run!("tests/expected/two.txt.n+0.out", TWO, "-n", "+0")
}

#[test]
fn two_n_plus_1() -> Result<()> {
    run!("tests/expected/two.txt.n+1.out", TWO, "-n", "+1")
}

#[test]
fn two_n_plus_2() -> Result<()> {
    run!("tests/expected/two.txt.n+2.out", TWO, "-n", "+2")
}

#[test]
fn two_c3() -> Result<()> {
    run!("tests/expected/two.txt.c3.out", TWO, "-c", "3")
}

#[test]
fn two_c_minus_3() -> Result<()> {
    run!("tests/expected/two.txt.c3.out", TWO, "-c=-3")
}

#[test]
fn two_c8() -> Result<()> {
    run!("tests/expected/two.txt.c8.out", TWO, "-c", "8")
}

#[test]
fn two_c_minus_8() -> Result<()> {
    run!("tests/expected/two.txt.c8.out", TWO, "-c=8")
}

#[test]
fn two_c12() -> Result<()> {
    run!("tests/expected/two.txt.c12.out", TWO, "-c", "12")
}

#[test]
fn two_c_minus_12() -> Result<()> {
    run!("tests/expected/two.txt.c12.out", TWO, "-c=-12")
}

#[test]
fn two_c200() -> Result<()> {
    run!("tests/expected/two.txt.c200.out", TWO, "-c", "200")
}

#[test]
fn two_c_minus_200() -> Result<()> {
    run!("tests/expected/two.txt.c200.out", TWO, "-c=-200")
}

#[test]
fn two_c_plus_0() -> Result<()> {
    run!("tests/expected/two.txt.c+0.out", TWO, "-c", "+0")
}

#[test]
fn two_c_plus_1() -> Result<()> {
    run!("tests/expected/two.txt.c+1.out", TWO, "-c", "+1")
}

#[test]
fn two_c_plus_2() -> Result<()> {
    run!("tests/expected/two.txt.c+2.out", TWO, "-c", "+2")
}

// --------------------------------------------------
#[test]
fn three() -> Result<()> {
    run!("tests/expected/three.txt.out", THREE)
}

#[test]
fn three_n0() -> Result<()> {
    run!("tests/expected/three.txt.n0.out", THREE, "-n", "0")
}

#[test]
fn three_n1() -> Result<()> {
    run!("tests/expected/three.txt.n1.out", THREE, "-n", "1")
}

#[test]
fn three_n_minus_1() -> Result<()> {
    run!("tests/expected/three.txt.n1.out", THREE, "-n=-1")
}

#[test]
fn three_n3() -> Result<()> {
    run!("tests/expected/three.txt.n3.out", THREE, "-n", "3")
}

#[test]
fn three_n_minus_3() -> Result<()> {
    run!("tests/expected/three.txt.n3.out", THREE, "-n=-3")
}

#[test]
fn three_n4() -> Result<()> {
    run!("tests/expected/three.txt.n4.out", THREE, "-n", "4")
}

#[test]
fn three_n_minus_4() -> Result<()> {
    run!("tests/expected/three.txt.n4.out", THREE, "-n=-4")
}

#[test]
fn three_n200() -> Result<()> {
    run!("tests/expected/three.txt.n200.out", THREE, "-n", "200")
}

#[test]
fn three_n_minus_200() -> Result<()> {
    run!("tests/expected/three.txt.n200.out", THREE, "-n=-200")
}

#[test]
fn three_n_plus_0() -> Result<()> {
    run!("tests/expected/three.txt.n+0.out", THREE, "-n", "+0")
}

#[test]
fn three_n_plus_1() -> Result<()> {
    run!("tests/expected/three.txt.n+1.out", THREE, "-n", "+1")
}

#[test]
fn three_n_plus_2() -> Result<()> {
    run!("tests/expected/three.txt.n+2.out", THREE, "-n", "+2")
}

#[test]
fn three_c3() -> Result<()> {
    run!("tests/expected/three.txt.c3.out", THREE, "-c", "3")
}

#[test]
fn three_c_minus_3() -> Result<()> {
    run!("tests/expected/three.txt.c3.out", THREE, "-c=-3")
}

#[test]
fn three_c8() -> Result<()> {
    run!("tests/expected/three.txt.c8.out", THREE, "-c", "8")
}

#[test]
fn three_c_minus_8() -> Result<()> {
    run!("tests/expected/three.txt.c8.out", THREE, "-c=8")
}

#[test]
fn three_c12() -> Result<()> {
    run!("tests/expected/three.txt.c12.out", THREE, "-c", "12")
}

#[test]
fn three_c_minus_12() -> Result<()> {
    run!("tests/expected/three.txt.c12.out", THREE, "-c=-12")
}

#[test]
fn three_c200() -> Result<()> {
    run!("tests/expected/three.txt.c200.out", THREE, "-c", "200")
}

#[test]
fn three_c_minus_200() -> Result<()> {
    run!("tests/expected/three.txt.c200.out", THREE, "-c=-200")
}

#[test]
fn three_c_plus_0() -> Result<()> {
    run!("tests/expected/three.txt.c+0.out", THREE, "-c", "+0")
}

#[test]
fn three_c_plus_1() -> Result<()> {
    run!("tests/expected/three.txt.c+1.out", THREE, "-c", "+1")
}

#[test]
fn three_c_plus_2() -> Result<()> {
    run!("tests/expected/three.txt.c+2.out", THREE, "-c", "+2")
}

// --------------------------------------------------
#[test]
fn twelve() -> Result<()> {
    run!("tests/expected/twelve.txt.out", TWELVE)
}

#[test]
fn twelve_n0() -> Result<()> {
    run!("tests/expected/twelve.txt.n0.out", TWELVE, "-n", "0")
}

#[test]
fn twelve_n1() -> Result<()> {
    run!("tests/expected/twelve.txt.n1.out", TWELVE, "-n", "1")
}

#[test]
fn twelve_n_minus_1() -> Result<()> {
    run!("tests/expected/twelve.txt.n1.out", TWELVE, "-n=-1")
}

#[test]
fn twelve_n3() -> Result<()> {
    run!("tests/expected/twelve.txt.n3.out", TWELVE, "-n", "3")
}

#[test]
fn twelve_n_minus_3() -> Result<()> {
    run!("tests/expected/twelve.txt.n3.out", TWELVE, "-n=-3")
}

#[test]
fn twelve_n4() -> Result<()> {
    run!("tests/expected/twelve.txt.n4.out", TWELVE, "-n", "4")
}

#[test]
fn twelve_n_minus_4() -> Result<()> {
    run!("tests/expected/twelve.txt.n4.out", TWELVE, "-n=-4")
}

#[test]
fn twelve_n200() -> Result<()> {
    run!("tests/expected/twelve.txt.n200.out", TWELVE, "-n", "200")
}

#[test]
fn twelve_n_minus_200() -> Result<()> {
    run!("tests/expected/twelve.txt.n200.out", TWELVE, "-n=-200")
}

#[test]
fn twelve_c3() -> Result<()> {
    run!("tests/expected/twelve.txt.c3.out", TWELVE, "-c", "3")
}

#[test]
fn twelve_c_minus_3() -> Result<()> {
    run!("tests/expected/twelve.txt.c3.out", TWELVE, "-c=-3")
}

#[test]
fn twelve_c8() -> Result<()> {
    run!("tests/expected/twelve.txt.c8.out", TWELVE, "-c", "8")
}

#[test]
fn twelve_c_minus_8() -> Result<()> {
    run!("tests/expected/twelve.txt.c8.out", TWELVE, "-c=8")
}

#[test]
fn twelve_c12() -> Result<()> {
    run!("tests/expected/twelve.txt.c12.out", TWELVE, "-c", "12")
}

#[test]
fn twelve_c_minus_12() -> Result<()> {
    run!("tests/expected/twelve.txt.c12.out", TWELVE, "-c=-12")
}

#[test]
fn twelve_c200() -> Result<()> {
    run!("tests/expected/twelve.txt.c200.out", TWELVE, "-c", "200")
}

#[test]
fn twelve_c_minus_200() -> Result<()> {
    run!("tests/expected/twelve.txt.c200.out", TWELVE, "-c=-200")
}

#[test]
fn twelve_n_plus_0() -> Result<()> {
    run!("tests/expected/twelve.txt.n+0.out", TWELVE, "-n", "+0")
}

#[test]
fn twelve_n_plus_1() -> Result<()> {
    run!("tests/expected/twelve.txt.n+1.out", TWELVE, "-n", "+1")
}

#[test]
fn twelve_n_plus_2() -> Result<()> {
    run!("tests/expected/twelve.txt.n+2.out", TWELVE, "-n", "+2")
}

#[test]
fn twelve_c_plus_0() -> Result<()> {
    run!("tests/expected/twelve.txt.c+0.out", TWELVE, "-c", "+0")
}

#[test]
fn twelve_c_plus_1() -> Result<()> {
    run!("tests/expected/twelve.txt.c+1.out", TWELVE, "-c", "+1")
}

#[test]
fn twelve_c_plus_2() -> Result<()> {
    run!("tests/expected/twelve.txt.c+2.out", TWELVE, "-c", "+2")
}

// --------------------------------------------------
#[test]
fn multiple_files() -> Result<()> {
    run!("tests/expected/all.out", TWELVE, EMPTY, ONE, THREE, TWO)
}

#[test]
fn multiple_files_n0() -> Result<()> {
    run!(
        "tests/expected/all.n0.out",
        "-n",
        "0",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n1() -> Result<()> {
    run!(
        "tests/expected/all.n1.out",
        "-n",
        "1",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n1_q() -> Result<()> {
    run!(
        "tests/expected/all.n1.q.out",
        "-n",
        "1",
        "-q",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n1_quiet() -> Result<()> {
    run!(
        "tests/expected/all.n1.q.out",
        "-n",
        "1",
        "--quiet",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n_minus_1() -> Result<()> {
    run!(
        "tests/expected/all.n1.out",
        "-n=-1",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n_plus_1() -> Result<()> {
    run!(
        "tests/expected/all.n+1.out",
        "-n",
        "+1",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n3() -> Result<()> {
    run!(
        "tests/expected/all.n3.out",
        "-n",
        "3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n_minus_3() -> Result<()> {
    run!(
        "tests/expected/all.n3.out",
        "-n=-3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_n_plus_3() -> Result<()> {
    run!(
        "tests/expected/all.n+3.out",
        "-n",
        "+3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_c0() -> Result<()> {
    run!(
        "tests/expected/all.c0.out",
        "-c",
        "0",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_c3() -> Result<()> {
    run!(
        "tests/expected/all.c3.out",
        "-c",
        "3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_c_minus_3() -> Result<()> {
    run!(
        "tests/expected/all.c3.out",
        "-c=-3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

#[test]
fn multiple_files_c_plus_3() -> Result<()> {
    run!(
        "tests/expected/all.c+3.out",
        "-c",
        "+3",
        TWELVE,
        EMPTY,
        ONE,
        THREE,
        TWO
    )
}

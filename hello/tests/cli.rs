use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn runs() {
    cargo_bin_cmd!()
        .assert()
        .success()
        .stdout("Hello, world!\n");
}

#[test]
fn true_ok() {
    cargo_bin_cmd!("true").assert().success();
}

#[test]
fn false_not_ok() {
    cargo_bin_cmd!("false").assert().failure();
}

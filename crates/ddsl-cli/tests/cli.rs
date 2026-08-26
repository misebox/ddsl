use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

const SAMPLE: &str = "../../examples/sample.ddsl";

fn ddsl() -> Command {
    Command::cargo_bin("ddsl").expect("バイナリ")
}

#[test]
fn sql_goes_to_stdout_by_default() {
    ddsl()
        .args(["sql", SAMPLE])
        .assert()
        .success()
        .stdout(predicate::str::contains("CREATE TABLE users"));
}

#[test]
fn output_option_writes_to_a_file() {
    let dir = tempdir();
    let out = dir.join("nested/schema.sql");
    ddsl()
        .args(["sql", SAMPLE, "-o"])
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let text = fs::read_to_string(&out).expect("生成された");
    assert!(text.contains("CREATE TABLE users"), "{text}");
}

#[test]
fn reads_stdin_when_input_is_a_dash() {
    ddsl()
        .args(["check", "-"])
        .write_stdin(fs::read_to_string(SAMPLE).expect("サンプル"))
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ok:"));
}

#[test]
fn deny_warnings_fails_on_a_warning() {
    let src = "table foo {\n  column id type=serial\n  pk id\n}\n";
    ddsl()
        .args(["check", "-"])
        .write_stdin(src)
        .assert()
        .success();
    ddsl()
        .args(["check", "-", "--deny-warnings"])
        .write_stdin(src)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--deny-warnings"));
}

#[test]
fn errors_go_to_stderr_and_exit_nonzero() {
    ddsl()
        .args(["sql", "-"])
        .write_stdin("table x {\n  column a type=nosuch\n}\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("postgres の型ではない"))
        .stdout(predicate::str::is_empty());
}

#[test]
fn unknown_dialect_is_reported() {
    ddsl()
        .args(["--dialect", "mysql", "check", SAMPLE])
        .assert()
        .failure()
        .stderr(predicate::str::contains("知らない dialect"));
}

fn tempdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ddsl-cli-test-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("一時ディレクトリ");
    dir
}

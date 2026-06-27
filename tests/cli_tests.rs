use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_no_args_prints_help() {
    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    cmd.arg("invalid_command_xyz")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_validate_no_panic() {
    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    // This might fail if mcp.spec.json is invalid or missing, but it shouldn't panic.
    let assert = cmd.arg("validate").assert();
    assert.code(predicate::in_iter(vec![0, 1]));
}

#[test]
fn test_cli_build_no_panic() {
    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    let assert = cmd.arg("build").assert();
    assert.code(predicate::in_iter(vec![0, 1]));
}

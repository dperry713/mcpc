use assert_cmd::Command;

#[test]
fn test_build_dry_run() {
    let mut clean_cmd = Command::cargo_bin("mcpc").unwrap();
    clean_cmd.arg("clean").assert().success();

    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    cmd.arg("build").arg("--dry-run");

    // Assuming we run in the crate root where a valid mcp.spec.json exists.
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("DRY RUN"));
}

#[test]
fn test_validate_success() {
    let mut cmd = Command::cargo_bin("mcpc").unwrap();
    cmd.arg("validate");
    cmd.assert().success();
}

//! CLI structure and global flags.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_commands() {
    Command::cargo_bin("paraclete")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("job"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("whoami"))
        .stdout(predicate::str::contains("token"));
}

#[test]
fn missing_subcommand_fails() {
    Command::cargo_bin("paraclete").unwrap().assert().failure().code(predicate::in_iter([1i32, 2]));
}

#[test]
fn global_json_flag_parses() {
    Command::cargo_bin("paraclete")
        .unwrap()
        .args(["--json", "health", "--help"])
        .assert()
        .success();
}

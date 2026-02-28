use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;

fn claudini() -> Command {
    cargo_bin_cmd!("claudini")
}

#[test]
fn version_flag() {
    claudini()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("claudini "));
}

#[test]
fn help_flag() {
    claudini()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI for switching Claude Code accounts"));
}

#[test]
fn profile_help() {
    claudini()
        .args(["profile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("use"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("rename"))
        .stdout(predicate::str::contains("current"));
}

#[test]
fn backup_help() {
    claudini()
        .args(["backup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("restore"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn profile_add_missing_name() {
    claudini()
        .args(["profile", "add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn profile_remove_missing_name() {
    claudini()
        .args(["profile", "remove"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn profile_rename_missing_args() {
    claudini()
        .args(["profile", "rename"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn backup_create_missing_name() {
    claudini()
        .args(["backup", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn backup_restore_missing_name() {
    claudini()
        .args(["backup", "restore"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn backup_delete_missing_name() {
    claudini()
        .args(["backup", "delete"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn json_flag_on_error() {
    claudini()
        .args(["--json", "profile", "current"])
        .env("HOME", "/tmp/claudini-test-nonexistent")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"error\""));
}

#[test]
fn unknown_subcommand_args_rejected() {
    claudini()
        .args(["someprofile", "extra", "args"])
        .assert()
        .failure();
}

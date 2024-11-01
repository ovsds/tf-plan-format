use crate::utils;
use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn default() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result = utils::get_test_data_file_contents("renders/tera/github_markdown.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");

    for file in utils::get_plan_files() {
        cmd.arg("--file").arg(file);
    }

    cmd.assert().success();
    cmd.assert().stdout(expected_result + "\n");
    cmd.assert().stderr("");
    cmd.assert().code(0);

    Ok(())
}

#[test]
fn invalid_files() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");

    cmd.arg("--file").arg("invalid");

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert()
        .stderr("Failed to parse plan. Failed to read file(invalid). No files found.\n");
    cmd.assert().code(64);

    Ok(())
}

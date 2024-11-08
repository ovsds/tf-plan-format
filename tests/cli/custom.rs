use crate::utils;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn default() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result = utils::get_test_data_file_contents("renders/tera/github_markdown.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    for file in utils::get_plan_files() {
        cmd.arg("--file").arg(file);
    }

    let template = tf_plan_format::template::tera::GITHUB_MARKDOWN_TEMPLATE;
    cmd.arg("--template").arg(template);

    cmd.assert().success();
    cmd.assert().stdout(expected_result + "\n");
    cmd.assert().stderr("");
    cmd.assert().code(0);

    Ok(())
}

#[test]
fn glob() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result = utils::get_test_data_file_contents("renders/tera/github_markdown.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    cmd.arg("--file").arg(format!(
        "{}/plans/*/terraform.tfplan.json",
        utils::TEST_DATA_FOLDER_PATH
    ));

    let template = tf_plan_format::template::tera::GITHUB_MARKDOWN_TEMPLATE;
    cmd.arg("--template").arg(template);

    cmd.assert().success();
    cmd.assert().stdout(expected_result + "\n");
    cmd.assert().stderr("");
    cmd.assert().code(0);

    Ok(())
}

#[test]
fn invalid_engine() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("invalid");

    let template = tf_plan_format::template::tera::GITHUB_MARKDOWN_TEMPLATE;
    cmd.arg("--template").arg(template);

    for file in utils::get_plan_files() {
        cmd.arg("--file").arg(file);
    }

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert()
        .stderr("Invalid engine(invalid). Invalid template engine: invalid\n");
    cmd.assert().code(64);

    Ok(())
}

#[test]
fn invalid_files() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    let template = tf_plan_format::template::tera::GITHUB_MARKDOWN_TEMPLATE;
    cmd.arg("--template").arg(template);

    cmd.arg("--file").arg("invalid");

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert()
        .stderr("Failed to parse plan. Failed to read file(invalid). No files found\n");
    cmd.assert().code(65);

    Ok(())
}

#[test]
fn test_invalid_template() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    let template = "{{invalid";
    cmd.arg("--template").arg(template);

    for file in utils::get_plan_files() {
        cmd.arg("--file").arg(file);
    }

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert().stderr(
        predicate::str::starts_with("Failed to render template. Failed to add template({{invalid). Failed to parse \'template\'")
    );
    cmd.assert().code(65);

    Ok(())
}

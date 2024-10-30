use assert_cmd::prelude::*;
use std::process::Command;

const TEST_DATA_FOLDER_PATH: &str = "tests/data";

#[must_use]
fn get_test_data_file_path(relative_path: &str) -> std::string::String {
    let folder = std::path::Path::new(TEST_DATA_FOLDER_PATH);
    let file = folder.join(relative_path);
    return file.to_string_lossy().to_string();
}

#[must_use]
fn get_test_data_file_contents(relative_path: &str) -> std::string::String {
    let file = get_test_data_file_path(relative_path);
    return std::fs::read_to_string(file).unwrap();
}

#[must_use]
fn get_files() -> Vec<std::string::String> {
    vec![
        get_test_data_file_path("plans/create/terraform.tfplan.json"),
        get_test_data_file_path("plans/delete/terraform.tfplan.json"),
        get_test_data_file_path("plans/delete-create/terraform.tfplan.json"),
        get_test_data_file_path("plans/no-op/terraform.tfplan.json"),
        get_test_data_file_path("plans/no-resources/terraform.tfplan.json"),
        get_test_data_file_path("plans/update/terraform.tfplan.json"),
    ]
}

#[test]
fn default() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result = get_test_data_file_contents("renders/tera/github_markdown.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    for file in get_files() {
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
fn invalid_engine() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("invalid");

    let template = tf_plan_format::template::tera::GITHUB_MARKDOWN_TEMPLATE;
    cmd.arg("--template").arg(template);

    for file in get_files() {
        cmd.arg("--file").arg(file);
    }

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert().stderr("Invalid engine(invalid).\n");
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
    cmd.assert().stderr("Failed to parse plan. Failed to read file(invalid). No such file or directory (os error 2)\n");
    cmd.assert().code(64);

    Ok(())
}

#[test]
fn test_invalid_template() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("custom");
    cmd.arg("--engine").arg("tera");

    let template = "{{invalid";
    cmd.arg("--template").arg(template);

    for file in get_files() {
        cmd.arg("--file").arg(file);
    }

    cmd.assert().failure();
    cmd.assert().stdout("");
    cmd.assert().stderr("Failed to render template. Failed to add template({{invalid). Failed to parse \'template\'\n");
    cmd.assert().code(64);

    Ok(())
}

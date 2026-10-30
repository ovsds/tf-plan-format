use crate::utils;
use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn default() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result =
        utils::get_test_data_file_contents("tera/renders/github_markdown/default.md");

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
fn changed() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result =
        utils::get_test_data_file_contents("tera/renders/github_markdown/show_changed_values.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");
    cmd.arg("--changed-values");

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
fn show_no_op() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result =
        utils::get_test_data_file_contents("tera/renders/github_markdown/show_no_op.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");
    cmd.arg("--show-no-op");

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
fn show_read() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result =
        utils::get_test_data_file_contents("tera/renders/github_markdown/show_read.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");
    cmd.arg("--show-read");

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
fn max_length() -> Result<(), Box<dyn std::error::Error>> {
    let expected_result =
        utils::get_test_data_file_contents("tera/renders/github_markdown/max_length.md");

    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("github");
    cmd.arg("--max-length").arg("4000");

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
        .stderr("Failed to parse plan. Failed to read file(invalid). No files found\n");
    cmd.assert().code(65);

    Ok(())
}

#[test]
fn helm_values_and_document_outputs() -> Result<(), Box<dyn std::error::Error>> {
    for changed in [false, true] {
        let mut cmd = Command::cargo_bin("tf_plan_format")?;
        cmd.arg("github")
            .arg("--file")
            .arg(utils::get_test_data_file_path("plans/artificial/helm.json"));
        if changed {
            cmd.arg("--changed-values");
        }
        let expected = utils::get_test_data_file_contents(if changed {
            "tera/renders/github_markdown/helm_changed.txt"
        } else {
            "tera/renders/github_markdown/helm.txt"
        });
        cmd.assert().success().stdout(expected + "\n").stderr("");
    }
    Ok(())
}

#[test]
fn helm_values_respect_length_limits() -> Result<(), Box<dyn std::error::Error>> {
    for changed in [false, true] {
        for max_length in [200, 350, 500, 800] {
            let mut cmd = Command::cargo_bin("tf_plan_format")?;
            cmd.arg("github")
                .arg("--file")
                .arg(utils::get_test_data_file_path("plans/artificial/helm.json"))
                .arg("--max-length")
                .arg(max_length.to_string());
            if changed {
                cmd.arg("--changed-values");
            }
            let output = cmd.output()?;
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let rendered = String::from_utf8(output.stdout)?;
            assert!(
                rendered.strip_suffix('\n').unwrap().len() <= max_length,
                "{rendered}"
            );
            assert_eq!(
                rendered.matches("<details>").count(),
                rendered.matches("</details>").count()
            );
            assert_eq!(
                rendered.matches("```diff").count(),
                rendered.lines().filter(|line| *line == "```").count()
            );
            assert!(!rendered.contains("\\n"), "{rendered}");
        }
    }
    Ok(())
}

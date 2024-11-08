use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn default() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;

    cmd.assert().failure();
    cmd.assert().code(64);
    cmd.assert().stdout("");
    cmd.assert()
        .stderr("No command provided. entity not found\n");

    Ok(())
}

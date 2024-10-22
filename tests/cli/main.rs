use assert_cmd::prelude::*;
use std::process::Command;

mod none;

#[test]
fn test_none() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;
    cmd.arg("placeholder");

    cmd.assert().success();
    cmd.assert().stdout("");
    cmd.assert().stderr("");
    cmd.assert().code(0);

    Ok(())
}

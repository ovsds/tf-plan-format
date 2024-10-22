use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_none() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("tf_plan_format")?;

    cmd.assert().failure();
    cmd.assert().code(64);
    cmd.assert().stdout("STDOUT\n");
    cmd.assert().stderr("STDERR\nNo command provided\n");

    Ok(())
}

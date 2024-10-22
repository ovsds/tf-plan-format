use crate::utils;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Placeholder command")]
    Placeholder {},
}

/// # Errors
/// Returns an error if the command is not provided
/// Returns subcommand errors
pub fn root(
    command: &Option<Commands>,
    stdout: impl std::io::Write,
    stderr: impl std::io::Write,
) -> Result<(), utils::cli::CommandError> {
    match command {
        Some(Commands::Placeholder {}) => {
            return Ok(());
        }
        None => {
            return none(stdout, stderr);
        }
    }
}

fn none(
    mut stdout: impl std::io::Write,
    mut stderr: impl std::io::Write,
) -> Result<(), utils::cli::CommandError> {
    writeln!(stdout, "STDOUT").unwrap();
    writeln!(stderr, "STDERR").unwrap();
    return Err(utils::cli::CommandError {
        message: "No command provided",
        exit_code: exitcode::USAGE,
    });
}

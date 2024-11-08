use clap::Parser;
use std::io::Write;

fn main() {
    let cli = tf_plan_format::cli::Cli::parse();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    match tf_plan_format::cli::root(&cli.command, &mut stdout, &mut stderr) {
        Ok(()) => {}
        Err(e) => {
            writeln!(stderr, "{e}").unwrap();
            match e.error_type {
                tf_plan_format::types::ErrorType::Default => std::process::exit(exitcode::USAGE),
                tf_plan_format::types::ErrorType::Command { exit_code } => {
                    std::process::exit(exit_code)
                }
            }
        }
    }
}

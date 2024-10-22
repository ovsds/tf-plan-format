use clap::Parser;
use std::io::Write;

fn main() {
    let cli = tf_plan_format::cli::Cli::parse();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    match tf_plan_format::cli::root(&cli.command, &mut stdout, &mut stderr) {
        Ok(()) => {}
        Err(e) => {
            writeln!(stderr, "{}", e.message).unwrap();
            std::process::exit(e.exit_code);
        }
    }
}

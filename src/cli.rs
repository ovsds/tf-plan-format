use crate::template;
use crate::tf;
use crate::types;
use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    #[command(about = "Render template with advanced options")]
    Custom {
        #[clap(
            short,
            long,
            help = "Template engine to be used, possible options: [tera].",
            default_value = "tera"
        )]
        engine: String,
        #[clap(
            short,
            long,
            help="File path or glob with terraform.tfplan.json, can be used multiple times.",
            num_args = 1..,
        )]
        file: Vec<String>,
        #[clap(short, long, help = "Template string")]
        template: String,
    },
    #[command(about = "Render into Github markdown")]
    Github {
        #[clap(
            short,
            long,
            help="File path or glob with terraform.tfplan.json, can be used multiple times.",
            num_args = 1..
        )]
        file: Vec<String>,
        #[clap(
            short,
            long,
            help = "Wheather to render changed values",
            default_value = "false"
        )]
        changed_values: bool,
    },
}

/// # Errors
/// Returns an error if the command is not provided
/// Returns subcommand errors
pub fn root(
    command: &Option<Commands>,
    stdout: impl std::io::Write,
    stderr: impl std::io::Write,
) -> Result<(), types::Error> {
    match command {
        Some(Commands::Custom {
            engine,
            file,
            template,
        }) => custom(engine, template, file, stdout),
        Some(Commands::Github {
            file,
            changed_values,
        }) => github(file, *changed_values, stdout),
        None => none(stdout, stderr),
    }
}

fn custom(
    engine: &str,
    template: &str,
    files: &[String],
    mut stdout: impl std::io::Write,
) -> Result<(), types::Error> {
    let engine = template::Engine::from_str(engine).map_err(|e| {
        types::Error::command(format!("Invalid engine({engine})"), exitcode::USAGE, e)
    })?;

    let data = tf::Data::from_files(files).map_err(|e| {
        types::Error::command("Failed to parse plan".to_string(), exitcode::DATAERR, e)
    })?;

    let result = template::render(&engine, &data, template).map_err(|e| {
        types::Error::command(
            "Failed to render template".to_string(),
            exitcode::DATAERR,
            e,
        )
    })?;

    writeln!(stdout, "{result}").unwrap();

    Ok(())
}

fn github(
    files: &[String],
    show_changed_values: bool,
    mut stdout: impl std::io::Write,
) -> Result<(), types::Error> {
    let data = tf::Data::from_files(files).map_err(|e| {
        types::Error::command("Failed to parse plan".to_string(), exitcode::DATAERR, e)
    })?;

    // Should never fail as the template is hardcoded
    let result = template::render_github(&data, show_changed_values).map_err(|e| {
        types::Error::command(
            "Failed to render template".to_string(),
            exitcode::DATAERR,
            e,
        )
    })?;

    writeln!(stdout, "{result}").unwrap();

    Ok(())
}

fn none(
    mut _stdout: impl std::io::Write,
    mut _stderr: impl std::io::Write,
) -> Result<(), types::Error> {
    Err(types::Error::command(
        "No command provided".to_string(),
        exitcode::USAGE,
        std::io::Error::from(std::io::ErrorKind::NotFound),
    ))
}

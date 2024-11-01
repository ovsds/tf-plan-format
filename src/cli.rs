use crate::template;
use crate::tf;
use crate::utils;
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
    },
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
        Some(Commands::Custom {
            engine,
            file,
            template,
        }) => custom(engine, template, file, stdout),
        Some(Commands::Github { file }) => github(file, stdout),
        None => none(stdout, stderr),
    }
}

fn custom(
    engine: &str,
    template: &str,
    files: &[String],
    mut stdout: impl std::io::Write,
) -> Result<(), utils::cli::CommandError> {
    let engine = template::Engine::from_str(engine).map_err(|()| utils::cli::CommandError {
        message: format!("Invalid engine({engine})."),
        exit_code: exitcode::USAGE,
    })?;

    let data = tf::Data::from_files(files).map_err(|e| utils::cli::CommandError {
        message: format!("Failed to parse plan. {}", e.message),
        exit_code: exitcode::USAGE,
    })?;

    let result =
        template::render(&engine, &data, template).map_err(|e| utils::cli::CommandError {
            message: format!("Failed to render template. {}", e.message),
            exit_code: exitcode::USAGE,
        })?;

    writeln!(stdout, "{result}").unwrap();

    Ok(())
}

fn github(
    files: &[String],
    mut stdout: impl std::io::Write,
) -> Result<(), utils::cli::CommandError> {
    let engine = template::Engine::Tera;
    let template = template::tera::GITHUB_MARKDOWN_TEMPLATE;

    let data = tf::Data::from_files(files).map_err(|e| utils::cli::CommandError {
        message: format!("Failed to parse plan. {}", e.message),
        exit_code: exitcode::USAGE,
    })?;

    // Should never fail as the template is hardcoded
    let result = template::render(&engine, &data, template).unwrap();

    writeln!(stdout, "{result}").unwrap();

    Ok(())
}

fn none(
    mut _stdout: impl std::io::Write,
    mut _stderr: impl std::io::Write,
) -> Result<(), utils::cli::CommandError> {
    Err(utils::cli::CommandError {
        message: "No command provided".to_string(),
        exit_code: exitcode::USAGE,
    })
}

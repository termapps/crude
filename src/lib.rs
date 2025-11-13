use anstream::eprintln;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;
use owo_colors::OwoColorize;
use proc_exit::Code;

use crate::{
    commands::Subcommands,
    error::{Result, exit},
};

mod db;
mod migration;

pub mod error;
mod styles;

pub mod commands;

pub use migration::planner::PlanOptions;

/// Migration toolkit for databases
#[derive(Debug, Parser)]
#[clap(name = "crude", version)]
#[command(styles = styles::styles())]
pub struct App {
    #[command(subcommand)]
    pub cmd: Subcommands,

    #[command(flatten)]
    pub color: Color,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(flatten)]
    pub options: Options,
}

#[derive(Debug, Parser)]
#[clap(next_help_heading = "Global Options")]
pub struct Options {
    /// Database URL
    #[arg(short, long, env = "DATABASE_URL")]
    pub url: Option<String>,

    /// Directory containing migrations
    #[arg(
        short = 'd',
        long,
        default_value = "./db/migrations",
        env = "MIGRATIONS_DIR"
    )]
    pub migrations_dir: Option<String>,

    /// File to dump the schema to
    #[arg(short, long, env = "SCHEMA_FILE")]
    pub schema: Option<String>,
}

impl App {
    pub fn run(self) -> Result {
        self.cmd.run(&self.options)
    }

    pub fn new(cmd: Subcommands, options: Options) -> Self {
        App {
            cmd,
            color: Color::default(),
            verbose: Verbosity::default(),
            options,
        }
    }
}

impl Options {
    /// Get the database URL or error out if not provided
    pub fn get_url(&self) -> Result<&str> {
        if let Some(url) = self.url.as_ref() {
            return Ok(url);
        }

        eprintln!(
            "{} the following required arguments were not provided:\n  {}\n\n{} {}{}\n\nFor more information, try '{}'.",
            "error:".red().bold(),
            "--url <URL>".green(),
            "Usage:".yellow(),
            "crude --url ".green(),
            "<URL> <COMMAND>".cyan(),
            "--help".green(),
        );

        exit(Code::FAILURE);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn verify_app() {
        App::command().debug_assert();
    }
}

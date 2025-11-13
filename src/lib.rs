use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;

use crate::{commands::Subcommands, error::Result};

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
    pub url: String,

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
        self.cmd.run(&self)
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

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn verify_app() {
        App::command().debug_assert();
    }
}

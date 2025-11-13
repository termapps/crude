use anstream::println;
use chrono::Utc;
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;
use tracing::instrument;

use crate::{App, error::Result, migration::dir::get_migrations_dir};

/// Create a new migration
#[derive(Debug, Parser)]
pub struct New {
    /// The name of the migration
    pub name: String,
}

impl New {
    #[instrument(name = "new", skip_all)]
    pub(crate) fn run(&self, opts: &App) -> Result {
        let migrations_dir = get_migrations_dir(opts);

        migrations_dir.check()?;

        let timestamp = Utc::now();

        if self.name == "rollup" || self.name == "init" {
            return Err(eyre!("migration name cannot be 'rollup' or 'init'"));
        }

        let compound_name = format!("{}_{}", timestamp.format("%Y%m%d%H%M%S"), self.name);

        migrations_dir.create_migration(&compound_name, None, None)?;

        println!("{} {}", "Created".green(), compound_name);

        Ok(())
    }
}

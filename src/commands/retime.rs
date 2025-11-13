use anstream::println;
use chrono::Utc;
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;
use tracing::instrument;

use crate::{App, error::Result, migration::dir::get_migrations_dir};

/// Regenerate the timestamp of a migration
#[derive(Debug, Parser)]
pub struct Retime {
    /// The name of the migration
    pub name: String,
}

impl Retime {
    #[instrument(name = "retime", skip_all)]
    pub(crate) fn run(&self, opts: &App) -> Result {
        let migrations_dir = get_migrations_dir(opts);
        let local = migrations_dir.load()?;

        let migration = local
            .into_iter()
            .find(|m| m.compound_name == self.name || m.name == self.name)
            .ok_or_else(|| eyre!("unable to find local migration {}", self.name))?;

        let timestamp = Utc::now();
        let compound_name = format!("{}_{}", timestamp.format("%Y%m%d%H%M%S"), migration.name);

        migrations_dir.rename_migration(&migration.compound_name, &compound_name)?;

        println!("{} {}", "Retimed".magenta(), migration.compound_name);

        Ok(())
    }
}

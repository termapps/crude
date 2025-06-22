use anstream::println;
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;
use tracing::instrument;

use crate::{App, db::get_db_adapter, error::Result, migration::dir::get_migrations_dir};

/// Repair a variant migration by updating its hash
#[derive(Debug, Parser)]
pub struct Repair {
    /// The name of the migration
    name: String,
}

impl Repair {
    #[instrument(name = "repair", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        let migrations_dir = get_migrations_dir(opts);
        let local = migrations_dir.load()?;

        let mut db = get_db_adapter(opts)?;

        let migration = local
            .into_iter()
            .find(|m| m.compound_name == self.name || m.name == self.name)
            .ok_or_else(|| eyre!("unable to find local migration {}", self.name))?;

        db.update_migration_hash(&migration.compound_name, &migration.hash)?;

        println!("{} {}", "Repaired".purple(), migration.compound_name);

        Ok(())
    }
}

use anstream::println;
use clap::Parser;
use tracing::instrument;

use crate::{Options, db::get_db_adapter, error::Result, migration::dir::get_migrations_dir};

/// Verify a migration by applying up, down, then up again
#[derive(Debug, Parser)]
pub struct Verify {
    /// The name of the migration
    pub name: Option<String>,
}

impl Verify {
    #[instrument(name = "verify", skip_all)]
    pub(crate) fn run(&self, opts: &Options) -> Result {
        let migrations_dir = get_migrations_dir(opts);
        let mut local = migrations_dir.load()?;

        let mut db = get_db_adapter(opts, true)?;

        if let Some(ref n) = self.name {
            local.retain(|m| &m.compound_name == n);
        }

        for m in local {
            println!("Verifying {}...", m.compound_name);

            db.run_up_migration(&m)?;
            db.run_down_migration(&m)?;
            db.run_up_migration(&m)?;

            println!(" OK");
        }

        Ok(())
    }
}

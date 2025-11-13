use std::process::Command;

use anstream::println;
use chrono::Utc;
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;
use sha2::{Digest, Sha256};
use tracing::instrument;

use crate::{
    Options,
    db::get_db_adapter,
    error::Result,
    migration::{
        dir::get_migrations_dir,
        planner::{MigrationState, Planner},
    },
};

/// Squash all applied migrations into a single baseline migration
#[derive(Debug, Parser)]
pub struct Rollup {}

impl Rollup {
    #[instrument(name = "rollup", skip_all)]
    pub(crate) fn run(&self, opts: &Options) -> Result {
        let migrations_dir = get_migrations_dir(opts);
        let local = migrations_dir.load()?;

        let mut db = get_db_adapter(opts, false)?;

        // Error out if status is not clean
        if Planner::new(opts)?
            .status()?
            .iter()
            .any(|s| s.state != MigrationState::Applied)
        {
            return Err(eyre!(
                "cannot rollup when there are pending, variant, or divergent migrations"
            ));
        }

        let url = &opts.url;

        // Dump current schema excluding the migrations table
        let up_sql = if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            let out = Command::new("pg_dump")
                .arg("--schema-only")
                .arg("--no-owner")
                .arg("--no-privileges")
                .arg("--exclude-schema=crude")
                .arg(format!("--dbname={url}"))
                .output()?;

            String::from_utf8_lossy(&out.stdout).into_owned()
        } else {
            let out = Command::new("sqlite3").arg(url).arg(".schema").output()?;

            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.contains("crude_migrations"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Build baseline migration
        let ts = Utc::now();
        let compound_name = format!("{}_rollup", ts.format("%Y%m%d%H%M%S"));

        let mut hasher = Sha256::new();
        hasher.update(up_sql.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // Dump data-only SQL for seed (exclude migrations table)
        let seed_sql = if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            let out = Command::new("pg_dump")
                .arg("--data-only")
                .arg("--inserts")
                .arg("--no-owner")
                .arg("--no-privileges")
                .arg("--exclude-schema=crude")
                .arg(format!("--dbname={url}"))
                .output()?;

            String::from_utf8_lossy(&out.stdout).into_owned()
        } else {
            let out = Command::new("sqlite3").arg(url).arg(".dump").output()?;

            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.contains("crude_migrations"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // Create the rollup migration
        migrations_dir.create_migration(&compound_name, Some(&up_sql), Some(&seed_sql))?;

        // Reset migration history and apply baseline record
        db.clear_migrations()?;
        db.record_baseline(&compound_name, &hash)?;

        // Remove all other migrations including previous rollup
        local
            .iter()
            .filter(|m| m.name != "init")
            .try_for_each(|m| {
                println!("{} {}", "Rolled up".cyan(), m.compound_name);

                migrations_dir.remove_migration(&m.compound_name)
            })?;

        Ok(())
    }
}

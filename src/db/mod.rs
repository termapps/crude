use std::{fs::write, process::Command};

use ::postgres::{Client, NoTls};
use eyre::eyre;
use rusqlite::Connection;
use tracing::debug;

use crate::{App, error::Result, migration::Migration};

mod postgres;
mod sqlite;

pub use postgres::PostgresAdapter;
pub use sqlite::SqliteAdapter;

/// Trait that defines database operations for migrations.
pub trait DatabaseAdapter {
    /// SQL to initialize the migrations tracking table.
    fn init_up_sql(&self) -> &'static str;

    /// Load applied migrations from the database.
    fn load_migrations(&mut self) -> Result<Vec<Migration>>;

    /// Run an UP migration and record it.
    fn run_up_migration(&mut self, migration: &Migration) -> Result<()>;

    /// Run a DOWN migration and remove it.
    fn run_down_migration(&mut self, migration: &Migration) -> Result<()>;

    /// Update the hash of a migration.
    fn update_migration_hash(&mut self, name: &str, hash: &str) -> Result<()>;

    /// Clear all recorded migrations from the tracking table.
    fn clear_migrations(&mut self) -> Result<()>;

    /// Record a baseline migration in the tracking table without executing its SQL.
    fn record_baseline(&mut self, name: &str, hash: &str) -> Result<()>;
}

/// Build a boxed DatabaseAdapter (Postgres or SQLite) based on the URL.
pub fn get_db_adapter(opts: &App) -> Result<Box<dyn DatabaseAdapter>> {
    let url = &opts.options.url;

    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        let client = Client::connect(url, NoTls)?;

        Ok(Box::new(PostgresAdapter::new(client)))
    } else if url.starts_with("sqlite://") {
        let conn = Connection::open(url)?;

        Ok(Box::new(SqliteAdapter::new(conn)))
    } else {
        Err(eyre!("unsupported database URL: {}", url))
    }
}

/// If the user specified a schema file, dump to it
pub fn maybe_dump_schema(opts: &App) -> Result<()> {
    if let Some(path) = &opts.options.schema {
        let url = &opts.options.url;

        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            // Use external pg_dump for schema-only dump
            let output = Command::new("pg_dump")
                .arg("--schema-only")
                .arg("--no-owner")
                .arg("--no-privileges")
                .arg(format!("--dbname={url}"))
                .output()?;

            write(path, &output.stdout)?;
        } else if url.starts_with("sqlite://") {
            // SQLite schema via sqlite3 .schema
            let output = Command::new("sqlite3").arg(url).arg(".schema").output()?;

            write(path, &output.stdout)?;
        }

        debug!("schema dumped to {path}");
    }

    Ok(())
}

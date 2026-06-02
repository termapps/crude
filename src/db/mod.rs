use std::{fs::write, thread::sleep, time::Duration};

use ::postgres::{Client, NoTls};
use eyre::eyre;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use rusqlite::Connection;
use tracing::{debug, trace};

use crate::{Options, error::Result, migration::Migration};

mod postgres;
mod sqlite;

pub use postgres::PostgresAdapter;
pub use sqlite::SqliteAdapter;

const IGNORED_PARAMS: &[&str] = &["pool_timeout", "connection_limit"];

pub fn strip_non_libpq_params(url: &str) -> String {
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        return url.to_string();
    }

    let Some(pos) = url.find('?') else {
        return url.to_string();
    };

    let (base, query) = url.split_at(pos);
    let query = &query[1..];

    let params = query
        .split('&')
        .filter(|p| {
            let key = p.split('=').next().unwrap_or("");

            !IGNORED_PARAMS.contains(&key)
        })
        .collect::<Vec<_>>();

    if params.is_empty() {
        base.to_string()
    } else {
        format!("{}?{}", base, params.join("&"))
    }
}

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

    /// Dump the database schema and return the output.
    fn dump_schema(&mut self, url: &str, exclude_migrations: bool) -> Result<Vec<u8>>;

    /// Dump the database data and return the output.
    fn dump_data(&mut self, url: &str, exclude_migrations: bool) -> Result<Vec<u8>>;
}

/// Build a boxed DatabaseAdapter (Postgres or SQLite) based on the URL.
pub fn get_db_adapter(opts: &Options, wait: bool) -> Result<Box<dyn DatabaseAdapter>> {
    let url = opts.get_url()?;

    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        let mut attempts = 0;

        let tls = MakeTlsConnector::new(TlsConnector::builder().build()?);

        let client = loop {
            let client = if url.contains("sslmode=require") {
                Client::connect(&url, tls.clone())
            } else {
                Client::connect(&url, NoTls)
            };

            match client {
                Ok(client) => break client,
                Err(err) => {
                    attempts += 1;

                    if !wait || attempts > 60 {
                        return Err(err.into());
                    }

                    trace!("failed to connect to postgres, retrying...");
                    sleep(Duration::from_secs(1));
                }
            }
        };

        Ok(Box::new(PostgresAdapter::new(client)))
    } else if url.starts_with("sqlite://") {
        let conn = Connection::open(url)?;

        Ok(Box::new(SqliteAdapter::new(conn)))
    } else {
        Err(eyre!("unsupported database URL: {}", url))
    }
}

/// If the user specified a schema file, dump to it
pub fn maybe_dump_schema(db: &mut Box<dyn DatabaseAdapter>, opts: &Options) -> Result<()> {
    if let Some(path) = &opts.schema {
        let url = opts.get_url()?;
        let schema = db.dump_schema(&url, false)?;

        write(path, &schema)?;

        debug!("schema dumped to {path}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_no_query() {
        assert_eq!(
            strip_non_libpq_params("postgres://user@host/db"),
            "postgres://user@host/db"
        );
    }

    #[test]
    fn strip_all_params_removed() {
        assert_eq!(
            strip_non_libpq_params("postgres://user@host/db?pool_timeout=30"),
            "postgres://user@host/db"
        );
    }

    #[test]
    fn strip_first_param() {
        assert_eq!(
            strip_non_libpq_params("postgres://user@host/db?pool_timeout=30&sslmode=require"),
            "postgres://user@host/db?sslmode=require"
        );
    }

    #[test]
    fn strip_last_param() {
        assert_eq!(
            strip_non_libpq_params("postgres://user@host/db?sslmode=require&pool_timeout=30"),
            "postgres://user@host/db?sslmode=require"
        );
    }

    #[test]
    fn strip_middle_param() {
        assert_eq!(
            strip_non_libpq_params(
                "postgres://user@host/db?sslmode=require&pool_timeout=30&connect_timeout=5"
            ),
            "postgres://user@host/db?sslmode=require&connect_timeout=5"
        );
    }

    #[test]
    fn strip_no_unknown_params() {
        assert_eq!(
            strip_non_libpq_params("postgres://user@host/db?sslmode=require"),
            "postgres://user@host/db?sslmode=require"
        );
    }

    #[test]
    fn strip_multiple_unknown() {
        assert_eq!(
            strip_non_libpq_params(
                "postgres://user@host/db?pool_timeout=30&connection_limit=5&sslmode=require"
            ),
            "postgres://user@host/db?sslmode=require"
        );
    }

    #[test]
    fn strip_sqlite_unchanged() {
        assert_eq!(
            strip_non_libpq_params("sqlite://db.sqlite3?pool_timeout=30"),
            "sqlite://db.sqlite3?pool_timeout=30"
        );
    }

    #[test]
    fn strip_postgresql_prefix() {
        assert_eq!(
            strip_non_libpq_params("postgresql://user@host/db?pool_timeout=30"),
            "postgresql://user@host/db"
        );
    }
}

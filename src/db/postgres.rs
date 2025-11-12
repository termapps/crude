use std::process::Command;

use postgres::Client;

use crate::{db::DatabaseAdapter, error::Result, migration::Migration};

/// Adapter for Postgres-backed migrations.
pub struct PostgresAdapter {
    client: Client,
}

impl PostgresAdapter {
    /// Wrap a `postgres::Client` as a migrator.
    pub fn new(client: Client) -> Self {
        PostgresAdapter { client }
    }
}

impl DatabaseAdapter for PostgresAdapter {
    fn init_up_sql(&self) -> &'static str {
        INIT_UP_SQL
    }

    fn load_migrations(&mut self) -> Result<Vec<Migration>> {
        let table_exists = self.client.query(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_schema = 'crude'
                AND table_name = 'migrations'
            )",
            &[],
        )?;

        if !table_exists.first().map(|row| row.get(0)).unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut migrations = Vec::new();

        let rows = self.client.query(
            "SELECT name, hash, down_sql FROM crude.migrations ORDER BY id ASC",
            &[],
        )?;

        for row in rows {
            let name: String = row.get(0);
            let hash: String = row.get(1);
            let down_sql: Option<String> = row.get(2);

            migrations.push(Migration::from_db(name, hash, down_sql)?);
        }

        Ok(migrations)
    }

    fn run_up_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.compound_name;
        let hash = &migration.hash;
        let up_sql = migration.up_sql.as_ref().unwrap();
        let down_sql = migration.down_sql.as_deref();
        let seed_sql = migration.seed_sql.as_deref();

        // Detect top-of-file marker to disable transaction
        let disable_tx = up_sql
            .trim_start()
            .to_lowercase()
            .starts_with("-- no-transaction");

        if disable_tx {
            // run up outside a transaction
            self.client.batch_execute(up_sql)?;
            self.client.execute(
                "INSERT INTO crude.migrations (name, hash, down_sql) VALUES ($1, $2, $3)",
                &[name, hash, &down_sql],
            )?;
        } else {
            // run up + record inside a transaction
            let mut tx = self.client.transaction()?;
            tx.batch_execute(up_sql)?;
            tx.execute(
                "INSERT INTO crude.migrations (name, hash, down_sql) VALUES ($1, $2, $3)",
                &[name, hash, &down_sql],
            )?;
            tx.commit()?;
        }

        // always run seed in its own transaction if provided
        if let Some(seed) = seed_sql {
            let mut tx = self.client.transaction()?;
            tx.batch_execute(seed)?;
            tx.commit()?;
        }

        Ok(())
    }

    fn run_down_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.compound_name;
        let down_sql = migration.down_sql.as_ref().unwrap();

        // Detect no-transaction marker
        let disable_tx = down_sql
            .trim_start()
            .to_lowercase()
            .starts_with("-- no-transaction");

        if disable_tx {
            self.client.batch_execute(down_sql)?;
            self.client
                .execute("DELETE FROM crude.migrations WHERE name = $1", &[name])?;
        } else {
            let mut tx = self.client.transaction()?;
            tx.batch_execute(down_sql)?;
            tx.execute("DELETE FROM crude.migrations WHERE name = $1", &[name])?;
            tx.commit()?;
        }

        Ok(())
    }

    fn update_migration_hash(&mut self, name: &str, hash: &str) -> Result<()> {
        self.client.execute(
            "UPDATE crude.migrations SET hash = $1 WHERE name = $2",
            &[&hash, &name],
        )?;

        Ok(())
    }

    fn clear_migrations(&mut self) -> Result<()> {
        self.client.batch_execute(
            "DELETE FROM crude.migrations WHERE id > (SELECT MIN(id) FROM crude.migrations);",
        )?;

        Ok(())
    }

    fn record_baseline(&mut self, name: &str, hash: &str) -> Result<()> {
        self.client.execute(
            "INSERT INTO crude.migrations (name, hash) VALUES ($1, $2)",
            &[&name, &hash],
        )?;

        Ok(())
    }

    fn dump_schema(&mut self, url: &str) -> Result<Vec<u8>> {
        // Use external pg_dump for schema-only dump
        let output = Command::new("pg_dump")
            .arg("--schema-only")
            .arg("--no-owner")
            .arg("--no-privileges")
            .arg(format!("--dbname={url}"))
            .output()?;

        Ok(output.stdout)
    }
}

/// DDL for creating the migrations table in Postgres.
pub const INIT_UP_SQL: &str = "\
CREATE SCHEMA crude;

CREATE TABLE crude.migrations (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    name VARCHAR(255) NOT NULL,
    hash VARCHAR(255) NOT NULL,
    down_sql TEXT,
    UNIQUE (name)
);
";

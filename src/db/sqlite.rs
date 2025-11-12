use std::process::Command;

use rusqlite::{Connection, params};

use crate::{db::DatabaseAdapter, error::Result, migration::Migration};

/// Adapter for SQLite-backed migrations.
pub struct SqliteAdapter {
    conn: Connection,
}

impl SqliteAdapter {
    /// Wrap a `rusqlite::Connection` as a migrator.
    pub fn new(conn: Connection) -> Self {
        SqliteAdapter { conn }
    }
}

impl DatabaseAdapter for SqliteAdapter {
    fn init_up_sql(&self) -> &'static str {
        INIT_UP_SQL
    }

    fn load_migrations(&mut self) -> Result<Vec<Migration>> {
        // Check if the crude_migrations table exists
        let table_exists = self
            .conn
            .prepare(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crude_migrations'",
            )
            .and_then(|mut stmt| {
                let count: i64 = stmt.query_row(params![], |row| row.get(0))?;
                Ok(count > 0)
            })?;

        if !table_exists {
            return Ok(Vec::new());
        }

        let mut stmt = self
            .conn
            .prepare("SELECT name, hash, down_sql FROM crude_migrations ORDER BY id ASC")?;

        let rows = stmt.query_map(params![], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        let mut migrations = Vec::new();

        for row in rows {
            let (name, hash, down_sql) = row?;
            migrations.push(Migration::from_db(name, hash, down_sql)?);
        }

        Ok(migrations)
    }

    fn run_up_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.compound_name;
        let hash = &migration.hash;
        let up_sql = migration.up_sql.as_ref().unwrap();
        let down_sql = migration.down_sql.as_deref().unwrap_or("");
        let seed_sql = migration.seed_sql.as_deref();

        // Check for no-transaction marker
        let disable_tx = up_sql
            .trim_start()
            .to_lowercase()
            .starts_with("-- no-transaction");

        if disable_tx {
            // run up outside a transaction
            self.conn.execute_batch(up_sql)?;
            self.conn.execute(
                "INSERT INTO crude_migrations (name, hash, down_sql) VALUES (?1, ?2, ?3)",
                params![name, hash, down_sql],
            )?;
        } else {
            // run up + record inside a transaction
            let tx = self.conn.transaction()?;
            tx.execute_batch(up_sql)?;
            tx.execute(
                "INSERT INTO crude_migrations (name, hash, down_sql) VALUES (?1, ?2, ?3)",
                params![name, hash, down_sql],
            )?;
            tx.commit()?;
        }

        // always run seed in its own transaction if provided
        if let Some(seed) = seed_sql {
            let tx = self.conn.transaction()?;
            tx.execute_batch(seed)?;
            tx.commit()?;
        }

        Ok(())
    }

    fn run_down_migration(&mut self, migration: &Migration) -> Result<()> {
        let name = &migration.compound_name;
        let down_sql = migration.down_sql.as_ref().unwrap();

        // Check for no-transaction marker
        let disable_tx = down_sql
            .trim_start()
            .to_lowercase()
            .starts_with("-- no-transaction");

        if disable_tx {
            self.conn.execute_batch(down_sql)?;
            self.conn.execute(
                "DELETE FROM crude_migrations WHERE name = ?1",
                params![name],
            )?;
        } else {
            let tx = self.conn.transaction()?;
            tx.execute_batch(down_sql)?;
            tx.execute(
                "DELETE FROM crude_migrations WHERE name = ?1",
                params![name],
            )?;
            tx.commit()?;
        }
        Ok(())
    }

    fn update_migration_hash(&mut self, name: &str, hash: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE crude_migrations SET hash = ?1 WHERE name = ?2",
            params![hash, name],
        )?;

        Ok(())
    }

    fn clear_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM crude.migrations WHERE id > (SELECT MIN(id) FROM crude.migrations);",
        )?;

        Ok(())
    }

    fn record_baseline(&mut self, name: &str, hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO crude_migrations (name, hash) VALUES (?1, ?2)",
            params![name, hash],
        )?;

        Ok(())
    }

    fn dump_schema(&mut self, url: &str) -> Result<Vec<u8>> {
        // SQLite schema via sqlite3 .schema
        let output = Command::new("sqlite3").arg(url).arg(".schema").output()?;

        Ok(output.stdout)
    }
}

/// DDL for creating the migrations table in SQLite.
pub const INIT_UP_SQL: &str = "\
CREATE TABLE crude_migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    name TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    down_sql TEXT
);
";

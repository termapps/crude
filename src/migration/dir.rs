use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    fs::{create_dir_all, read_dir, remove_dir_all, rename, write},
    path::PathBuf,
};

use eyre::eyre;

use crate::{Options, error::Result, migration::Migration};

/// Manages filesystem operations for local migrations.
pub struct MigrationsDir {
    pub dir: PathBuf,
}

impl Display for MigrationsDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.dir.display())
    }
}

impl MigrationsDir {
    /// Create a handler rooted at the given migrations directory.
    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        Self { dir: dir.into() }
    }

    /// Ensure the migrations directory exists.
    pub fn check(&self) -> Result<()> {
        if !self.dir.exists() {
            return Err(eyre!(
                "migrations directory does not exist: {}, use `crude init` to create it",
                self.dir.display()
            ));
        }

        Ok(())
    }

    /// Create the migrations directory for the first time.
    pub fn create(&self) -> Result<()> {
        // Error if the migrations directory already exists
        if self.dir.exists() {
            return Err(eyre!(
                "migrations directory already exists: {}",
                self.dir.display()
            ));
        }

        create_dir_all(&self.dir)?;

        Ok(())
    }

    /// Load local migrations from subdirectories (sorted by name).
    pub fn load(&self) -> Result<Vec<Migration>> {
        self.check()?;

        let mut migrations = Vec::new();

        let mut dirs = read_dir(&self.dir)?
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|ft| ft.is_dir()))
            .collect::<Vec<_>>();

        dirs.sort_by_key(|e| e.file_name());

        for entry in dirs {
            let mig = Migration::from_dir(&entry.path())?;
            migrations.push(mig);
        }

        Ok(migrations)
    }

    /// Write a new migration folder and blank SQL files.
    pub fn create_migration(
        &self,
        compound_name: &String,
        up_sql: Option<&str>,
        seed_sql: Option<&str>,
    ) -> Result<()> {
        let path = self.dir.join(compound_name);

        create_dir_all(&path)?;

        write(path.join("up.sql"), up_sql.unwrap_or_default())?;
        write(path.join("down.sql"), "")?;
        write(path.join("seed.sql"), seed_sql.unwrap_or_default())?;

        Ok(())
    }

    /// Remove a migration by its compound name.
    pub fn remove_migration(&self, compound_name: &String) -> Result<()> {
        let path = self.dir.join(compound_name);

        remove_dir_all(&path)?;

        Ok(())
    }

    /// Rename a migration
    pub fn rename_migration(&self, from: &String, to: &String) -> Result<()> {
        let from_path = self.dir.join(from);
        let to_path = self.dir.join(to);

        rename(from_path, to_path)?;

        Ok(())
    }
}

/// Build a MigrationsDir from CLI options.
pub fn get_migrations_dir(opts: &Options) -> MigrationsDir {
    let dir = opts.migrations_dir.as_deref().unwrap_or("./db/migrations");

    MigrationsDir::new(dir)
}

use std::{fs::read_to_string, path::Path};

use chrono::{DateTime, NaiveDateTime, Utc};
use eyre::eyre;
use sha2::{Digest, Sha256};

use crate::error::Result;

pub mod dir;
pub mod planner;

/// Represents a migration, either loaded locally or from the database.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Short name of the migration, e.g. "create_users".
    pub name: String,
    /// Compound name, e.g. "20230623041234_create_users".
    pub compound_name: String,
    /// Contents of the `up.sql`, if available.
    pub up_sql: Option<String>,
    /// Contents of the `down.sql`, if available.
    pub down_sql: Option<String>,
    /// Contents of the `seed.sql`, if available.
    pub seed_sql: Option<String>,
    /// SHA256 hash (hex) of the `up.sql`, if available.
    pub hash: String,
}

impl Migration {
    fn from_compound_name(compound_name: &String) -> Result<(String, DateTime<Utc>)> {
        let underscore = compound_name
            .find('_')
            .ok_or_else(|| eyre!("invalid migration name (missing '_'): {}", compound_name))?;

        let name = compound_name[underscore + 1..].to_string();
        let ts_str = &compound_name[..underscore];

        let timestamp = NaiveDateTime::parse_from_str(ts_str, "%Y%m%d%H%M%S")
            .map_err(|e| eyre!("failed to parse timestamp {}: {}", ts_str, e))?
            .and_utc();

        Ok((name, timestamp))
    }

    /// Load a local migration from a directory.
    pub fn from_dir(path: &Path) -> Result<Self> {
        let compound_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| eyre!("invalid migration directory {}", path.display()))?
            .to_string();

        let (name, _) = Self::from_compound_name(&compound_name)?;

        let up_path = path.join("up.sql");
        let up_sql = read_to_string(&up_path)
            .map_err(|e| eyre!("unable to read migration {}: {}", up_path.display(), e))?;

        let mut hasher = Sha256::new();
        hasher.update(up_sql.as_bytes());
        let hash = hex::encode(hasher.finalize());

        let down_sql = read_to_string(path.join("down.sql"))
            .ok()
            .filter(|s| !s.is_empty());

        let seed_sql = read_to_string(path.join("seed.sql"))
            .ok()
            .filter(|s| !s.is_empty());

        Ok(Migration {
            name,
            compound_name,
            up_sql: Some(up_sql),
            down_sql,
            seed_sql,
            hash,
        })
    }

    /// Construct a migration record from database metadata.
    pub fn from_db(compound_name: String, hash: String, down_sql: Option<String>) -> Result<Self> {
        let (name, _) = Self::from_compound_name(&compound_name)?;

        Ok(Migration {
            name,
            compound_name,
            up_sql: None,
            down_sql,
            seed_sql: None,
            hash,
        })
    }
}

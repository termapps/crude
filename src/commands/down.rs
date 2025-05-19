use clap::Parser;
use tracing::instrument;

use crate::{error::Result, App};

/// Rollback the last migration
#[derive(Debug, Parser)]
pub struct Down {
    /// The number of migrations to rollback
    #[clap(short, long, default_value = "1")]
    number: u32,

    /// Rollback all migrations
    #[clap(short, long, conflicts_with_all = &["number"])]
    all: bool,
}

impl Down {
    #[instrument(name = "down", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        Ok(())
    }
}

use clap::Parser;
use tracing::instrument;

use crate::{error::Result, App};

/// Rollback & re-apply the last migration
#[derive(Debug, Parser)]
pub struct Redo {
    /// The number of migrations to redo
    #[clap(short, long, default_value = "1")]
    number: u32,

    /// Redo all migrations
    #[clap(short, long, conflicts_with_all = &["number"])]
    all: bool,
}

impl Redo {
    #[instrument(name = "redo", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        Ok(())
    }
}

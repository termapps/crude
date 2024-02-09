use crate::error::Result;

use clap::Parser;
use tracing::instrument;

/// List all migrations and their status
#[derive(Debug, Parser)]
pub struct List {}

impl List {
    #[instrument(name = "list", skip_all)]
    pub fn run(self) -> Result {
        Ok(())
    }
}

use crate::error::Result;

use clap::Parser;
use tracing::instrument;

/// Create a new migration
#[derive(Debug, Parser)]
pub struct New {
    /// The name of the migration
    name: String,
}

impl New {
    #[instrument(name = "new", skip_all)]
    pub fn run(self) -> Result {
        Ok(())
    }
}

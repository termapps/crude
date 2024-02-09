use crate::error::Result;

use clap::Parser;
use tracing::instrument;

/// Apply all pending migrations
#[derive(Debug, Parser)]
pub struct Up {}

impl Up {
    #[instrument(name = "up", skip_all)]
    pub fn run(self) -> Result {
        Ok(())
    }
}

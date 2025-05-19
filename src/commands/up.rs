use clap::Parser;
use tracing::instrument;

use crate::{error::Result, App};

/// Apply all pending migrations
#[derive(Debug, Parser)]
pub struct Up {}

impl Up {
    #[instrument(name = "up", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        Ok(())
    }
}

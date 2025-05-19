use clap::Parser;
use tracing::instrument;

use crate::{error::Result, App};

/// List all migrations and their status
#[derive(Debug, Parser)]
pub struct List {}

impl List {
    #[instrument(name = "list", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        Ok(())
    }
}

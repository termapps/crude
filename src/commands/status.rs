use clap::Parser;
use tracing::instrument;

use crate::{
    App,
    error::Result,
    migration::planner::{Planner, print_status},
};

/// List all migrations and their status
#[derive(Debug, Parser)]
pub struct Status {}

impl Status {
    #[instrument(name = "status", skip_all)]
    pub(crate) fn run(&self, opts: &App) -> Result {
        let status = Planner::new(opts)?.status()?;

        print_status(&status);

        Ok(())
    }
}

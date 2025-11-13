use clap::Parser;
use tracing::instrument;

use crate::{
    App,
    db::{get_db_adapter, maybe_dump_schema},
    error::Result,
    migration::planner::{PlanOptions, Planner},
};

/// Apply all pending migrations
#[derive(Debug, Parser)]
pub struct Up {
    /// Number of migrations to apply
    #[clap(short, long)]
    pub number: Option<usize>,

    #[clap(flatten)]
    pub plan_options: PlanOptions,
}

impl Up {
    #[instrument(name = "up", skip_all)]
    pub(crate) fn run(&self, opts: &App) -> Result {
        let mut db = get_db_adapter(opts, true)?;

        Planner::new(opts)?
            .count(self.number)
            .up(&mut db)?
            .run(&mut db, &self.plan_options)?;

        maybe_dump_schema(&mut db, opts)?;

        Ok(())
    }
}

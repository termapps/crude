use clap::Parser;
use tracing::instrument;

use crate::{
    App,
    db::{get_db_adapter, maybe_dump_schema},
    error::Result,
    migration::planner::{PlanOptions, Planner},
};

/// Rollback all divergent and variant migrations, then apply all pending migrations
#[derive(Debug, Parser)]
pub struct Fix {
    #[clap(flatten)]
    plan_options: PlanOptions,
}

impl Fix {
    #[instrument(name = "fix", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        let mut db = get_db_adapter(opts, false)?;

        Planner::new(opts)?
            .fix()?
            .run(&mut db, &self.plan_options)?;

        maybe_dump_schema(opts)?;

        Ok(())
    }
}

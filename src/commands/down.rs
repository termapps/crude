use clap::Parser;
use tracing::instrument;

use crate::{
    Options,
    db::{get_db_adapter, maybe_dump_schema},
    error::Result,
    migration::planner::{PlanOptions, Planner},
};

/// Rollback the most recent migration
#[derive(Debug, Parser)]
pub struct Down {
    /// Number of migrations to rollback
    #[clap(short, long, default_value_t = 1, conflicts_with = "all")]
    pub number: usize,

    /// Rollback all applied migrations
    #[clap(short, long)]
    pub all: bool,

    /// Only show the migration plan without rolling back
    #[clap(short, long)]
    pub plan_only: bool,

    /// Ignore divergent migrations
    #[clap(long)]
    pub ignore_divergent: bool,

    /// Ignore unreversible migrations
    #[clap(long)]
    pub ignore_unreversible: bool,
}

impl Down {
    #[instrument(name = "down", skip_all)]
    pub(crate) fn run(&self, opts: &Options) -> Result {
        let mut db = get_db_adapter(opts, false)?;

        Planner::new(opts)?
            .set_ignore_divergent(self.ignore_divergent)
            .set_ignore_unreversible(self.ignore_unreversible)
            .count((!self.all).then_some(self.number))
            .down()?
            .run(
                &mut db,
                &PlanOptions {
                    seed: false,
                    plan_only: self.plan_only,
                },
            )?;

        maybe_dump_schema(&mut db, opts)?;

        Ok(())
    }
}

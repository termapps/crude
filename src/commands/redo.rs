use clap::Parser;
use tracing::instrument;

use crate::{
    App,
    db::{get_db_adapter, maybe_dump_schema},
    error::Result,
    migration::planner::{PlanOptions, Planner},
};

/// Rollback and re-apply the most recent migration
#[derive(Debug, Parser)]
pub struct Redo {
    /// Number of migrations to redo
    #[clap(short, long, default_value_t = 1, conflicts_with = "all")]
    pub number: usize,

    /// Redo all applied migrations
    #[clap(short, long)]
    pub all: bool,

    #[clap(flatten)]
    pub plan_options: PlanOptions,

    /// Ignore divergent migrations
    #[clap(long)]
    pub ignore_divergent: bool,

    /// Ignore unreversible migrations
    #[clap(long)]
    pub ignore_unreversible: bool,
}

impl Redo {
    #[instrument(name = "redo", skip_all)]
    pub(crate) fn run(&self, opts: &App) -> Result {
        let mut db = get_db_adapter(opts, false)?;

        Planner::new(opts)?
            .set_ignore_divergent(self.ignore_divergent)
            .set_ignore_unreversible(self.ignore_unreversible)
            .count((!self.all).then_some(self.number))
            .redo()?
            .run(&mut db, &self.plan_options)?;

        maybe_dump_schema(&mut db, opts)?;

        Ok(())
    }
}

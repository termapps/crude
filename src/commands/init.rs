use clap::Parser;
use tracing::{debug, instrument};

use crate::{
    App,
    db::{get_db_adapter, maybe_dump_schema},
    error::Result,
    migration::{
        dir::get_migrations_dir,
        planner::{PlanOptions, Planner},
    },
};

/// Initialize the migrations dir & database
#[derive(Debug, Parser)]
pub struct Init {}

impl Init {
    #[instrument(name = "init", skip_all)]
    pub fn run(&self, opts: &App) -> Result {
        let migrations_dir = get_migrations_dir(opts);

        migrations_dir.create()?;

        let mut db = get_db_adapter(opts, true)?;
        let up_sql = db.init_up_sql();

        let compound_name = String::from("20000101000000_init");

        migrations_dir.create_migration(&compound_name, Some(up_sql), None)?;

        debug!("created migrations directory {migrations_dir}");

        Planner::new(opts)?.up(&mut db)?.run(
            &mut db,
            &PlanOptions {
                seed: false,
                plan_only: false,
            },
        )?;

        maybe_dump_schema(&mut db, opts)?;

        Ok(())
    }
}

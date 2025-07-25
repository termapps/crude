use std::{
    cmp::min,
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
};

use anstream::{print, println};
use clap::Parser;
use eyre::eyre;
use owo_colors::OwoColorize;

use crate::{
    App,
    db::{DatabaseAdapter, get_db_adapter},
    error::Result,
    migration::{Migration, dir::get_migrations_dir},
};

/// The state of a migration when comparing local vs. database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationState {
    Applied,
    Pending,
    Variant,
    Divergent,
}

/// Status entry for a single migration.
#[derive(Debug, Clone)]
pub struct Status {
    pub state: MigrationState,
    pub migration: Migration,
}

/// A single step in a migration plan.
#[derive(Debug, Clone)]
pub enum PlanStep {
    Up(Migration),
    Down(Migration),
}

impl Display for PlanStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PlanStep::Up(m) => write!(f, "{:>4} - {}", "Up".green(), m.compound_name),
            PlanStep::Down(m) => write!(f, "{:>4} - {}", "Down".red(), m.compound_name),
        }
    }
}

#[derive(Debug, Parser)]
pub struct PlanOptions {
    /// Run seed.sql after applying migrations
    #[clap(long, env = "SEED")]
    pub seed: bool,

    /// Only show the migration plan without applying it
    #[clap(short, long)]
    pub plan_only: bool,
}

/// A plan of migrations to apply or rollback.
#[derive(Debug, Clone)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
}

impl Display for Plan {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (i, step) in self.steps.iter().enumerate() {
            writeln!(f, "{:>2}. {step}", i + 1)?;
        }

        Ok(())
    }
}

impl Plan {
    /// Run the plan with the given options
    pub fn run(&self, db: &mut Box<dyn DatabaseAdapter>, options: &PlanOptions) -> Result<()> {
        if options.plan_only {
            print!("{self}");
        } else {
            for step in &self.steps {
                match step.clone() {
                    PlanStep::Down(m) => db.run_down_migration(&m)?,
                    PlanStep::Up(mut m) => {
                        if !options.seed {
                            m.seed_sql = None;
                        }

                        db.run_up_migration(&m)?;
                    }
                }

                println!("{step}");
            }
        }

        Ok(())
    }
}

/// Builder for migration plans (`up`, `down`, `redo`, `fix`, `status`).
pub struct Planner {
    local: Vec<Migration>,
    remote: Vec<Migration>,
    count: Option<usize>,
    ignore_divergent: bool,
    ignore_unreversible: bool,
    local_map: HashMap<String, Migration>,
    remote_map: HashMap<String, Migration>,
}

impl Planner {
    /// Start a new plan builder.
    pub fn new(opts: &App) -> Result<Self> {
        let migrations_dir = get_migrations_dir(opts);
        let local = migrations_dir.load()?;

        let mut db = get_db_adapter(opts)?;
        let remote = db.load_migrations()?;

        let planner = Self {
            local: Vec::new(),
            remote: Vec::new(),
            count: Some(1),
            ignore_divergent: false,
            ignore_unreversible: false,
            local_map: HashMap::new(),
            remote_map: HashMap::new(),
        }
        .local_migrations(&local)
        .remote_migrations(&remote);

        Ok(planner)
    }

    pub fn local_migrations(mut self, migrations: &[Migration]) -> Self {
        self.local = migrations.to_vec();
        self.local_map = migrations
            .iter()
            .cloned()
            .map(|m| (m.compound_name.clone(), m))
            .collect();
        self
    }

    pub fn remote_migrations(mut self, migrations: &[Migration]) -> Self {
        self.remote = migrations.to_vec();
        self.remote_map = migrations
            .iter()
            .cloned()
            .map(|m| (m.compound_name.clone(), m))
            .collect();
        self
    }

    pub fn count(mut self, count: Option<usize>) -> Self {
        self.count = count;
        self
    }

    pub fn set_ignore_divergent(mut self, ignore_divergent: bool) -> Self {
        self.ignore_divergent = ignore_divergent;
        self
    }

    pub fn set_ignore_unreversible(mut self, ignore_unreversible: bool) -> Self {
        self.ignore_unreversible = ignore_unreversible;
        self
    }

    /// Build status listing for each migration.
    pub fn status(&self) -> Result<Vec<Status>> {
        let mut res = Vec::new();
        let mut i_local = 0;
        let mut i_remote = 0;

        while i_local < self.local.len() || i_remote < self.remote.len() {
            let status = if i_local < self.local.len() && i_remote < self.remote.len() {
                let local = &self.local[i_local];
                let remote = &self.remote[i_remote];

                if local.compound_name == remote.compound_name {
                    let state = if local.hash == remote.hash {
                        MigrationState::Applied
                    } else {
                        MigrationState::Variant
                    };

                    i_local += 1;
                    i_remote += 1;

                    Status {
                        state,
                        migration: local.clone(),
                    }
                } else if local.compound_name < remote.compound_name {
                    let migration = local.clone();

                    i_local += 1;

                    Status {
                        state: MigrationState::Pending,
                        migration,
                    }
                } else {
                    let migration = remote.clone();

                    i_remote += 1;

                    Status {
                        state: MigrationState::Divergent,
                        migration,
                    }
                }
            } else if i_local < self.local.len() {
                let migration = self.local[i_local].clone();

                i_local += 1;

                Status {
                    state: MigrationState::Pending,
                    migration,
                }
            } else {
                let migration = self.remote[i_remote].clone();

                i_remote += 1;

                Status {
                    state: MigrationState::Divergent,
                    migration,
                }
            };

            res.push(status);
        }

        Ok(res)
    }

    fn check_rollup(&self) -> Result<()> {
        if self
            .local
            .iter()
            .any(|m| m.name == "rollup" && !self.remote_map.contains_key(&m.compound_name))
        {
            return Err(eyre!(
                "unable to sync the rollup, please reset the database"
            ));
        }

        Ok(())
    }

    fn sync_rollup(mut self, db: &mut Box<dyn DatabaseAdapter>) -> Result<Self> {
        // Is there a pending rollup migration?
        if let Some(rollup) = self
            .local
            .iter()
            .find(|m| m.name == "rollup" && !self.remote_map.contains_key(&m.compound_name))
        {
            // If there's any pending migrations before the rollup, error out
            if self.local.iter().any(|m| {
                m.name != "init"
                    && m.name != "rollup"
                    && !self.remote_map.contains_key(&m.compound_name)
                    && m.compound_name < rollup.compound_name
            }) {
                return Err(eyre!(
                    "pending migrations before the rollup, please re-order them to the end"
                ));
            }

            // If there are any remote non-divergent migrations, error out
            if !self
                .remote
                .iter()
                .filter(|m| m.name != "init")
                .all(|m| !self.local_map.contains_key(&m.compound_name))
            {
                return Err(eyre!(
                    "unable to sync the rollup, please reset the database"
                ));
            }

            // Sync the rollup only if it's not during startup of database
            if !self.remote.is_empty() {
                db.clear_migrations()?;
                db.record_baseline(&rollup.compound_name, &rollup.hash)?;

                println!("{} - {}", "Sync".cyan(), rollup.compound_name);

                self = self.remote_migrations(&db.load_migrations()?);
            }
        }

        Ok(self)
    }

    /// Plan applying migrations (`up`).
    pub fn up(mut self, db: &mut Box<dyn DatabaseAdapter>) -> Result<Plan> {
        self = self.sync_rollup(db)?;

        let pending = self
            .local
            .iter()
            .filter(|m| !self.remote_map.contains_key(&m.compound_name))
            .cloned()
            .collect::<Vec<_>>();

        let to_do = self.count.unwrap_or(pending.len());
        let take = min(to_do, pending.len());

        let steps = pending.into_iter().take(take).map(PlanStep::Up).collect();

        Ok(Plan { steps })
    }

    /// Plan rolling back migrations (`down`).
    pub fn down(self) -> Result<Plan> {
        self.check_rollup()?;

        let applied = self
            .remote
            .iter()
            .filter(|m| !self.ignore_divergent || self.local_map.contains_key(&m.compound_name))
            .cloned()
            .rev()
            .collect::<Vec<_>>();

        let to_rollback = self.count.unwrap_or(applied.len());
        let take = min(to_rollback, applied.len());

        let applied = applied.into_iter().take(take).collect::<Vec<_>>();

        for m in &applied {
            if !self.ignore_unreversible && m.down_sql.is_none() {
                return Err(eyre!(
                    "unable to rollback unreversible migration {}",
                    m.compound_name
                ));
            }
        }

        let steps = applied.into_iter().map(PlanStep::Down).collect();

        Ok(Plan { steps })
    }

    /// Plan redoing the most recent migrations (`redo`).
    pub fn redo(self) -> Result<Plan> {
        self.check_rollup()?;

        let mut down_steps = Vec::new();
        let mut up_steps = Vec::new();

        let applied = self
            .remote
            .iter()
            .filter(|m| m.name != "init" && m.name != "rollup")
            .filter(|m| !self.ignore_divergent || self.local_map.contains_key(&m.compound_name))
            .filter(|m| !self.ignore_unreversible || m.down_sql.is_some())
            .cloned()
            .collect::<Vec<_>>();

        let count = self.count.unwrap_or(applied.len());

        let recent = applied.into_iter().rev().take(count).collect::<Vec<_>>();

        for m in recent {
            if m.down_sql.is_none() {
                return Err(eyre!(
                    "unable to redo unreversible migration {}",
                    m.compound_name
                ));
            }

            if let Some(local) = self.local_map.get(&m.compound_name) {
                down_steps.push(PlanStep::Down(m.clone()));
                up_steps.push(PlanStep::Up(local.clone()));
            } else {
                return Err(eyre!(
                    "unable to redo divergent migration {}",
                    m.compound_name
                ));
            }
        }

        Ok(Plan {
            steps: down_steps
                .into_iter()
                .chain(up_steps.into_iter().rev())
                .collect(),
        })
    }

    /// Plan fixing variant/divergent migrations and applying pending ones (`fix`).
    pub fn fix(self) -> Result<Plan> {
        self.check_rollup()?;

        // Find the index of the oldest divergent or variant migration in the database
        let index = self
            .remote
            .iter()
            .position(|m| {
                !self.local_map.contains_key(&m.compound_name)
                    || m.hash
                        != self
                            .local_map
                            .get(&m.compound_name)
                            .map(|l| l.hash.clone())
                            .unwrap_or_default()
            })
            .unwrap_or(self.remote.len());

        let mut steps = Vec::new();

        // Get all db migrations from the index onwards
        for m in self.remote.iter().skip(index).rev() {
            if m.down_sql.is_none() {
                return Err(eyre!(
                    "unable to rollback unreversible migration {}",
                    m.compound_name
                ));
            }

            steps.push(PlanStep::Down(m.clone()));
        }

        // Apply pending migrations
        for m in self.local.iter().filter(|l| {
            !self
                .remote
                .iter()
                .take(index)
                .any(|d| d.compound_name == l.compound_name)
        }) {
            steps.push(PlanStep::Up(m.clone()));
        }

        Ok(Plan { steps })
    }
}

/// Print the status of each migration (Applied, Pending, Variant, Divergent).
pub fn print_status(statuses: &[Status]) {
    for status in statuses.iter() {
        let label = match status.state {
            MigrationState::Applied => format!("{:>9}", "Applied".green()),
            MigrationState::Pending => format!("{:>9}", "Pending".yellow()),
            MigrationState::Variant => format!("{:>9}", "Variant".red()),
            MigrationState::Divergent => format!("{:>9}", "Divergent".red()),
        };

        println!("{label:<9} - {}", status.migration.compound_name);
    }
}

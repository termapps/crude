use crate::utils::Database;

pub struct PostgresDatabase {
    // pub connection: Connection,
}

impl Database for PostgresDatabase {
    fn name() -> &'static str {
        "PostgreSQL"
    }
}

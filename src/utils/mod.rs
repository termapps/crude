use crate::error::Result;

mod postgres;
mod simple;

pub trait Strategy {
    fn migrate(&self) -> Result;

    fn migrate_to(&self, version: u32) -> Result;
}

pub trait Database {
    fn name() -> &'static str;
}

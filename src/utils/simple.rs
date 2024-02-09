use crate::{error::Result, utils::Strategy};

/// Simple strategy for migrating databases
pub struct Simple;

impl Strategy for Simple {
    fn migrate(&self) -> Result {
        Ok(())
    }

    fn migrate_to(&self, version: u32) -> Result {
        Ok(())
    }
}

use clap::Parser;

use crate::{App, error::Result};

mod down;
mod fix;
mod init;
mod new;
mod redo;
mod repair;
mod retime;
mod rollup;
mod status;
mod up;
mod verify;

#[derive(Debug, Parser)]
pub enum Subcommands {
    Init(init::Init),
    New(new::New),
    Status(status::Status),
    Up(up::Up),
    Down(down::Down),
    Redo(redo::Redo),
    Fix(fix::Fix),
    Repair(repair::Repair),
    Rollup(rollup::Rollup),
    Retime(retime::Retime),
    #[clap(hide = true)]
    Verify(verify::Verify),
}

impl Subcommands {
    pub fn run(&self, opts: &App) -> Result {
        match self {
            Self::Init(x) => x.run(opts),
            Self::New(x) => x.run(opts),
            Self::Status(x) => x.run(opts),
            Self::Up(x) => x.run(opts),
            Self::Down(x) => x.run(opts),
            Self::Redo(x) => x.run(opts),
            Self::Fix(x) => x.run(opts),
            Self::Repair(x) => x.run(opts),
            Self::Rollup(x) => x.run(opts),
            Self::Retime(x) => x.run(opts),
            Self::Verify(x) => x.run(opts),
        }
    }
}

use clap::Parser;

use crate::{error::Result, App};

mod down;
mod list;
mod new;
mod redo;
mod up;

#[derive(Debug, Parser)]
pub enum Subcommands {
    List(list::List),
    Up(up::Up),
    Down(down::Down),
    Redo(redo::Redo),
    New(new::New),
}

impl Subcommands {
    pub fn run(&self, opts: &App) -> Result {
        match self {
            Self::List(x) => x.run(opts),
            Self::Up(x) => x.run(opts),
            Self::Down(x) => x.run(opts),
            Self::Redo(x) => x.run(opts),
            Self::New(x) => x.run(opts),
        }
    }
}

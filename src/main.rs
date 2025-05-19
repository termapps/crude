use std::io::stdout;

use anstream::{AutoStream, ColorChoice};
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use colorchoice_clap::Color;
use tracing_log::AsTrace;
use tracing_subscriber::prelude::*;

mod error;
mod styles;

mod utils;

mod commands;

/// Migration toolkit for databases
#[derive(Debug, Parser)]
#[clap(name = "crude", version)]
#[command(styles = styles::styles())]
struct App {
    #[command(subcommand)]
    cmd: commands::Subcommands,

    #[command(flatten)]
    color: Color,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    #[command(flatten)]
    options: Options,
}

#[derive(Debug, Parser)]
#[clap(next_help_heading = "Global Options")]
struct Options {
    /// Database URL
    #[arg(short, long, env = "DATABASE_URL")]
    url: String,

    /// Directory containing migration files
    #[arg(
        short = 'd',
        long,
        default_value = "./db/migrations",
        env = "MIGRATIONS_DIR"
    )]
    migrations_dir: Option<String>,
}

fn main() {
    let program = App::parse();

    program.color.write_global();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_ansi(!matches!(AutoStream::choice(&stdout()), ColorChoice::Never))
                .with_filter(program.verbose.log_level_filter().as_trace()),
        )
        .init();

    let result = program.cmd.run(&program);

    error::finish(result);
}

#[cfg(test)]
mod test {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn verify_app() {
        App::command().debug_assert();
    }
}

use clap::Parser;

/// Migration toolkit for databases
#[derive(Debug, Parser)]
#[clap(name = "crude")]
struct App {
    #[clap(subcommand)]
    cmd: Subcommands,
}

#[derive(Debug, Parser)]
enum Subcommands {}

fn main() {
    let program = App::parse();

    match program.cmd {
        // Subcommands::Pie(x) => x.run(),
    }
}

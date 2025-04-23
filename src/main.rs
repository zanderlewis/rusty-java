mod build;
mod commands;
mod config;
mod gradle;
mod run;
mod utils;

use clap::Parser;
use commands::Commands;
use utils::{printerr, separator};

#[derive(Parser)]
#[clap(name = "rsj", version = "0.1.0", author = "")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    separator();

    let result = match cli.command {
        Commands::Build => build::build_project(),
        Commands::Run => run::run_project(),
        Commands::Clean => build::clean_build(),
        Commands::Init => build::init_project(),
    };

    if let Err(e) = result {
        printerr(&e);
    }

    separator();
}

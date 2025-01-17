mod build;
mod buildtools;
mod commands;
mod config;
mod run;
mod utils;

use clap::Parser;
use commands::Commands;
use utils::{basic_seperator, printerr, rsj_seperator};

#[derive(Parser)]
#[clap(name = "rsj", version = "0.1.0", author = "")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    rsj_seperator();

    match cli.command {
        Commands::Build => {
            if let Err(e) = build::build_project() {
                printerr(&e);
            }
        }
        Commands::Run => {
            if let Err(e) = run::run_project() {
                printerr(&e);
            }
        }
        Commands::Clean => {
            if let Err(e) = build::clean_build() {
                printerr(&e);
            }
        }
        Commands::Init => {
            if let Err(e) = build::init_project() {
                printerr(&e);
            }
        }
    }
    basic_seperator();
}

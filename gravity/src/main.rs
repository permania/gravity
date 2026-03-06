mod cli;
use gravlib::initialize_db;

use clap::Parser;
use cli::args::{GravityArgs, GravityCommand, InitArg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = GravityArgs::parse();

    match cli.cmd {
        GravityCommand::Init(InitArg {db_name: name}) => initialize_db(name)?,
    }
    Ok(())
}


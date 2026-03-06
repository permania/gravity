mod cli;
use gravlib::initialize_db;

use clap::Parser;
use cli::args::{GravityCommand, GravityArgs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = GravityArgs::parse();

    match cli.cmd {
        GravityCommand::Init => initialize_db()?,
    }
    Ok(())
}


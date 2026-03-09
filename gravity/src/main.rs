mod cli;
use gravlib::{initialize_db, read_db};

use clap::Parser;
use cli::args::{DBArg, GravityArgs, GravityCommand};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = GravityArgs::parse();

    match cli.cmd {
        GravityCommand::Init(DBArg { db_name: name }) => initialize_db(name)?,
        GravityCommand::Read(DBArg { db_name: name }) => read_db(name)?,
    }
    Ok(())
}

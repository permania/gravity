mod cli;
use gravlib::{bin_db, compile_db, initialize_db, read_db, state_db};

use clap::Parser;
use cli::args::{DBArg, GravityArgs, GravityCommand};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = GravityArgs::parse();

    match cli.cmd {
        GravityCommand::Init(DBArg { db_name: name }) => initialize_db(name)?,
        GravityCommand::Read(DBArg { db_name: name }) => read_db(name)?,
        GravityCommand::Bin(DBArg { db_name: name }) => bin_db(name)?,
        GravityCommand::State(DBArg { db_name: name }) => state_db(name)?,
        GravityCommand::Compile(DBArg { db_name: name }) => compile_db(name)?,
    }
    
    Ok(())
}

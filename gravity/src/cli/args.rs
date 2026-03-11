use clap::{
    Args, Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct GravityArgs {
    #[clap(subcommand)]
    pub cmd: GravityCommand,
}

#[derive(Debug, Subcommand)]
pub enum GravityCommand {
    #[command()]
    /// Create a new database file
    Init(DBArg),

    #[command()]
    /// Read a database schema
    Read(DBArg),

    #[command()]
    /// Read a database schema to binary format
    Bin(DBArg),

    #[command()]
    /// Compile a database schema to binary format
    Compile(DBArg),

    #[command()]
    /// Read a database schema and print the State
    State(DBArg),

    #[command()]
    /// Decompile a database file to a schema that produces it
    Dump(DBArg),

    #[command()]
    /// Run the Gravity REPL
    Repl(DBArg),
}

#[derive(Debug, Args)]
pub struct DBArg {
    #[arg(default_value = "database")]
    pub db_name: String,
}

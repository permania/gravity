use std::fs::File;

mod error;
use error::GravityError;

mod parse;
use parse::{
    ast::{self, Rule},
    typecheck,
};
use pest::Parser;

const DB_EXT: &str = ".gravdb";
const SCM_EXT: &str = ".gravscm";

pub fn read_db(name: String) -> Result<(), GravityError> {
    let input = std::fs::read_to_string(name + SCM_EXT)?;
    let program = ast::parse_program(input);
    typecheck::run(program)?;

    // println!("{:#?}", program);
    Ok(())
}

pub fn initialize_db(name: String) -> Result<(), GravityError> {
    let _db = File::create_new(name + DB_EXT)?;
    Ok(())
}

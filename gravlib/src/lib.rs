use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

mod error;
use error::GravityError;

mod parse;
use indexmap::IndexMap;
use parse::{
    ast::{self, Rule},
    eval::{self, State},
    typecheck,
};
use pest::Parser;

const DB_EXT: &str = "gravdb";
const SCM_EXT: &str = "gravscm";
const MAGIC: [u8; 9] = [0xff, 0xfe, 0xc0, 0xaa, 0xab, b'g', b'r', b'a', b'v'];

fn read_db_state(name: String) -> Result<State, GravityError> {
    let fp = PathBuf::from(name);
    let file = File::open(&fp)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 9];
    reader.read_exact(&mut buffer)?;

    if buffer == MAGIC {
        let mut db_bytes = Vec::new();
        let _read = reader.read_to_end(&mut db_bytes)?;
        let db_state = postcard::from_bytes::<eval::State>(&db_bytes)?;
        Ok(db_state)
    } else {
	std::mem::drop(reader);
	let input = fs::read_to_string(&fp)?;
        let program = ast::parse_program(input);
        typecheck::run(program.clone())?;
        let db_state = eval::eval_program(program)?;
        Ok(db_state)
    }
}

pub fn compile_db(name: String) -> Result<(), GravityError> {
    let fp = PathBuf::from(&name);
    let db_state = read_db_state(name)?;
    let output = postcard::to_allocvec(&db_state)?;
    let mut final_bytes = MAGIC.to_vec();
    final_bytes.extend(output);
    fs::write(fp.with_extension(DB_EXT), final_bytes)?;

    Ok(())
}

pub fn bin_db(name: String) -> Result<(), GravityError> {
    let db_state = read_db_state(name)?;
    let output = postcard::to_allocvec(&db_state)?;
    let mut final_bytes = MAGIC.to_vec();
    final_bytes.extend(output);

    for byte in final_bytes {
        print!("{:02x} ", byte);
    }

    Ok(())
}

pub fn state_db(name: String) -> Result<(), GravityError> {
    let db_state = read_db_state(name)?;
    println!("{:#?}", db_state);

    Ok(())
}

#[allow(dead_code)]
pub fn read_db(name: String) -> Result<(), GravityError> {
    let db_state = read_db_state(name)?;
    let output = postcard::to_allocvec(&db_state)?;
    todo!("pretty reading");

    Ok(())
}

pub fn initialize_db(name: String) -> Result<(), GravityError> {
    let fp = PathBuf::from(name);
    let _db = File::create_new(&fp)?;
    Ok(())
}

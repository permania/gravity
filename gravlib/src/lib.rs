use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

pub mod error;
use error::GravityError;

mod parse;
use indexmap::IndexMap;
pub use parse::{
    ast::{self, Expr, Op, Rule},
    eval::{self, Assignment, State, Value},
    typecheck,
};
use pest::Parser;
use serde::de::value;

pub type GravityState = eval::State;

const DB_EXT: &str = "gravdb";
pub const SCM_EXT: &str = "gravscm";
const MAGIC: [u8; 9] = [0xff, 0xfe, 0xc0, 0xaa, 0xab, b'g', b'r', b'a', b'v'];

pub fn read_db_state(name: String) -> Result<State, GravityError> {
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

pub fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Decimal(d) => d.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Text(s) => format!("\"{}\"", s),
        Expr::Ident(n) => n.clone(),
        Expr::SelfRef => "%".to_string(),
        Expr::Negate(e) => format!("-{}", expr_to_string(e)),
        Expr::Factorial(e, n) => format!("{}{}", expr_to_string(e), "!".repeat(*n as usize)),
        Expr::BinOp(l, op, r) => format!(
            "({} {} {})",
            expr_to_string(l),
            op_to_string(op),
            expr_to_string(r)
        ),
    }
}

fn op_to_string(op: &Op) -> &str {
    match op {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "*",
        Op::Div => "/",
        Op::Mod => "&",
        Op::Pow => "^",
    }
}

fn create_assignment(a: Assignment) -> String {
    format!("{} {} = {};", a.typ, a.name, expr_to_string(&a.expr))
}

fn create_relationship(name: &str, expr: &Expr) -> String {
    format!("{} <- {};", name, expr_to_string(expr))
}

pub fn dump_db_state(name: String, state: &State) -> Result<(), GravityError> {
    let fp = PathBuf::from(&name);
    let mut lines = Vec::<String>::new();
    let db_state = state;

    for a in db_state.def.iter() {
        lines.push(create_assignment(a.to_owned()));
    }

    for pair in db_state.rel.iter() {
	let (name, expr) = &pair;
        lines.push(create_relationship(name, expr));
    }

    let content = lines.join("\n");

    fs::write(fp.with_extension(SCM_EXT), content)?;
    Ok(())
}

pub fn dump_db(name: String) -> Result<(), GravityError> {
    let fp = PathBuf::from(&name);
    let mut lines = Vec::<String>::new();
    let db_state = read_db_state(name)?;

    for a in db_state.def.into_iter() {
        lines.push(create_assignment(a));
    }

    for pair in db_state.rel.into_iter() {
	let (name, expr) = &pair;
        lines.push(create_relationship(name, expr));
    }

    let content = lines.join("\n");

    fs::write(fp.with_extension(SCM_EXT), content)?;
    Ok(())
}

pub fn initialize_db(name: String) -> Result<(), GravityError> {
    let fp = PathBuf::from(name);
    let _db = File::create_new(&fp)?;
    Ok(())
}

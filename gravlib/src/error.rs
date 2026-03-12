use std::{io, process};

use reedline_repl_rs::reedline;
use thiserror::Error;

use crate::parse::ast::Type;

#[derive(Debug, Error)]
pub enum GravityError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("use before declaration: {0}")]
    UndefinedVariable(String),

    #[error("duplicate declaration of variable: {0}")]
    Duplication(String),

    #[error("type mismatch in {0}: expected {1}, found {2}")]
    AssignmentMismatch(String, Type, Type),

    #[error("type mismatch in expression: {0} and {1} are incompatible")]
    TypeMismatch(Type, Type),

    #[error("cannot negate type: {0}")]
    InvalidNegation(Type),

    #[error("cannot take factorial of type: {0}")]
    InvalidFactorial(Type),

    #[error("serialization error: {0}")]
    SerializeError(#[from] postcard::Error),

    #[error("repl error: {0}")]
    ReplError(#[from] reedline_repl_rs::Error),

    #[error("failed to parse value: {0}")]
    ParseError(String),

    #[error("filepath has no extension")]
    NoExtension,

    #[error("pest error: {0}")]
    PestError(#[from] pest::error::Error<crate::Rule>),

    #[error("Unable to convert string to type")]
    InvalidString,

    #[error("Self Reference should only be used inside a relationship")]
    SelfRef,
}

pub fn handle_error(r: Result<(), GravityError>) -> ! {
    if let Err(e) = r {
        eprintln!("{e}");
        process::exit(1);
    } else {
        process::exit(0);
    }
}

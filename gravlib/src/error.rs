use std::{io, process};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GravityError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("Use before declaration: {0}")]
    UndefinedVariable(String),
}

pub fn handle_error(r: Result<(), GravityError>) -> ! {
    if let Err(e) = r {
        eprintln!("{e}");
        process::exit(1);
    } else {
        process::exit(0);
    }
}

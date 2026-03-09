use std::collections::HashSet;

use crate::error::GravityError;

use super::ast::{Program, Statement};

pub fn run(prg: Program) -> Result<(), GravityError> {
    let mut def = HashSet::<String>::new();

    for stmt in prg.slf {
        match stmt {
            Statement::Assignment { name, .. } => {
                def.insert(name);
            }
            Statement::Relationship { name, .. } => {
                if !def.contains(&name) {
		    return Err(GravityError::UndefinedVariable(name))
                };
            }
        }
    }

    Ok(())
}

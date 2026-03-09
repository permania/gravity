use std::collections::HashMap;

use crate::error::GravityError;

use super::ast::{Expr, Program, Statement, Type};

fn expr_type(expr: &Expr, def: &HashMap<String, Type>, name: &str) -> Result<Type, GravityError> {
    match expr {
        Expr::Number(_) => Ok(Type::Number),
        Expr::Decimal(_) => Ok(Type::Decimal),
        Expr::Text(_) => Ok(Type::Text),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Ident(n) => def
            .get(n)
            .cloned()
            .ok_or(GravityError::UndefinedVariable(n.clone())),
        Expr::BinOp(lhs, _, rhs) => {
            let l = expr_type(lhs, def, name)?;
            let r = expr_type(rhs, def, name)?;
            if l == r {
                return Ok(l)
            }
	    return Err(GravityError::TypeMismatch(r, l))
        }
        _ => todo!(),
    }
}

pub fn run(prg: Program) -> Result<(), GravityError> {
    let mut def = HashMap::<String, Type>::new();

    for stmt in prg.slf {
        match stmt {
            Statement::Assignment { typ, name, expr } => {
                let expr_t = expr_type(&expr, &def, &name)?;

                if typ != expr_t {
                    return Err(GravityError::AssignmentMismatch(name, typ, expr_t));
                }

                if def.insert(name.clone(), typ).is_some() {
                    return Err(GravityError::Duplication(name));
                }
            }
            Statement::Relationship { name, .. } => {
                if def.get(&name).is_none() {
                    return Err(GravityError::UndefinedVariable(name));
                };
            }
        }
    }

    Ok(())
}

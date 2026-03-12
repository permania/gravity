use std::collections::HashMap;

use crate::{State, error::GravityError};

use super::ast::{Expr, Program, Statement, Type};

pub fn expr_type_state(expr: &Expr, state: &State) -> Result<Type, GravityError> {
    match expr {
        Expr::Number(_) => Ok(Type::Number),
        Expr::Decimal(_) => Ok(Type::Decimal),
        Expr::Text(_) => Ok(Type::Text),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Negate(inner) => {
            let expr_t = expr_type_state(inner, state)?;
            match expr_t {
                Type::Number | Type::Decimal => Ok(expr_t),
                _ => Err(GravityError::InvalidNegation(expr_t)),
            }
        }
        Expr::Factorial(inner, _) => {
            let expr_t = expr_type_state(inner, state)?;
            match expr_t {
                Type::Number => Ok(expr_t),
                _ => Err(GravityError::InvalidFactorial(expr_t)),
            }
        }
        Expr::BinOp(lhs, _, rhs) => {
            let l = expr_type_state(lhs, state)?;
            let r = expr_type_state(rhs, state)?;
            if l == r {
                return Ok(l);
            }
            Err(GravityError::TypeMismatch(r, l))
        }
        Expr::Ident(n) => Ok(state
            .def
            .iter()
            .find(|d| d.name == *n)
            .cloned()
            .unwrap()
            .typ),
        _ => unreachable!(),
    }
}

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
                return Ok(l);
            }
            Err(GravityError::TypeMismatch(r, l))
        }
        Expr::SelfRef => def
            .get(name)
            .cloned()
            .ok_or(GravityError::UndefinedVariable(name.to_string())),
        Expr::Negate(inner) => {
            let expr_t = expr_type(inner, def, name)?;
            match expr_t {
                Type::Number | Type::Decimal => Ok(expr_t),
                _ => Err(GravityError::InvalidNegation(expr_t)),
            }
        }
        Expr::Factorial(inner, _) => {
            let expr_t = expr_type(inner, def, name)?;
            match expr_t {
                Type::Number => Ok(expr_t),
                _ => Err(GravityError::InvalidFactorial(expr_t)),
            }
        }
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
            Statement::Relationship { name, expr } => {
                let typ = match def.get(&name) {
                    None => return Err(GravityError::UndefinedVariable(name)),
                    Some(t) => t.clone(),
                };
                let expr_t = expr_type(&expr, &def, &name)?;
                if typ != expr_t {
                    return Err(GravityError::AssignmentMismatch(name, typ, expr_t));
                }
            }
        }
    }

    Ok(())
}

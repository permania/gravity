use std::collections::HashMap;

use crate::{State, ast::RecField, error::GravityError};

use super::ast::{Expr, Program, RecDef, Statement, Type};

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
        _ => unreachable!("bad"),
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

fn expr_type_insertion(expr: &Expr, def: &HashMap<String, Type>) -> Result<Type, GravityError> {
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
            let l = expr_type_insertion(lhs, def)?;
            let r = expr_type_insertion(rhs, def)?;
            if l == r {
                return Ok(l);
            }
            Err(GravityError::TypeMismatch(r, l))
        }
        Expr::Negate(inner) => {
            let expr_t = expr_type_insertion(inner, def)?;
            match expr_t {
                Type::Number | Type::Decimal => Ok(expr_t),
                _ => Err(GravityError::InvalidNegation(expr_t)),
            }
        }
        Expr::Factorial(inner, _) => {
            let expr_t = expr_type_insertion(inner, def)?;
            match expr_t {
                Type::Number => Ok(expr_t),
                _ => Err(GravityError::InvalidFactorial(expr_t)),
            }
        }
        _ => Err(GravityError::SelfRef)
    }
}

pub fn check_assignment(
    typ: &Type,
    name: &String,
    expr: &Expr,
    def: &mut HashMap<String, Type>,
) -> Result<(), GravityError> {
    let expr_t = expr_type(expr, def, name)?;

    if typ.to_owned() != expr_t {
        return Err(GravityError::AssignmentMismatch(
            name.to_owned(),
            typ.to_owned(),
            expr_t,
        ));
    }

    if def.insert(name.clone(), typ.to_owned()).is_some() {
        return Err(GravityError::Duplication(name.to_owned()));
    }

    Ok(())
}

pub fn check_relationship(
    name: &String,
    expr: &Expr,
    def: &mut HashMap<String, Type>,
) -> Result<(), GravityError> {
    let typ = match def.get(name) {
        None => return Err(GravityError::UndefinedVariable(name.to_owned())),
        Some(t) => t.clone(),
    };
    let expr_t = expr_type(expr, def, name)?;
    if typ != expr_t {
        return Err(GravityError::AssignmentMismatch(
            name.to_owned(),
            typ,
            expr_t,
        ));
    }

    Ok(())
}

pub fn check_insertion(
    target: &String,
    exprs: &Vec<Expr>,
    def: &mut Vec<RecDef>,
    vars: &HashMap<String, Type>
) -> Result<(), GravityError> {
    if !def.iter().any(|r| r.name == *target) {
        return Err(GravityError::UndefinedVariable(target.to_owned()));
    }

    let target_def = def.iter().find(|r| r.name == *target).cloned().unwrap();

    if exprs.len() != target_def.fields.len() {
        return Err(GravityError::WrongInsertionCount(
            target.to_owned(),
            target_def.fields.len(),
            exprs.len(),
        ));
    }

    for (expr, field) in exprs.iter().zip(target_def.fields.iter()) {
        let expr_t = expr_type_insertion(expr, vars)?;
        if expr_t != field.typ {
            return Err(GravityError::WrongInsertionType(
                target.to_owned(),
                field.name.clone(),
                field.typ.clone(),
                expr_t,
            ));
        }
    }

    Ok(())
}

pub fn run(prg: Program) -> Result<(), GravityError> {
    let mut def = HashMap::<String, Type>::new();
    let mut def_rec = Vec::<RecDef>::new();

    for recd in prg.rec {
        check_record_def(recd, &mut def_rec)?;
    }

    for stmt in prg.slf {
        match stmt {
            Statement::Assignment { typ, name, expr } => {
                check_assignment(&typ, &name, &expr, &mut def)?;
            }
            Statement::Relationship { name, expr } => {
                check_relationship(&name, &expr, &mut def)?;
            }
            Statement::Insertion { target, exprs } => {
                check_insertion(&target, &exprs, &mut def_rec, &def)?;
            }
        }
    }

    Ok(())
}

fn check_record_def(recd: RecDef, def: &mut Vec<RecDef>) -> Result<(), GravityError> {
    if def.iter().any(|r| r.name == recd.name) {
        return Err(GravityError::Duplication(recd.name));
    }

    let mut field_names = Vec::new();
    let mut key_count = 0;

    for field in recd.fields.iter() {
        if field.is_key {
            if key_count == 1 {
                return Err(GravityError::MultipleKeys(recd.name.clone()));
            }
            if matches!(field.typ, Type::Bool | Type::Decimal) {
                return Err(GravityError::WrongKey(recd.name.clone()));
            }

            key_count += 1;
        }

        if field_names.contains(&field.name) {
            return Err(GravityError::Duplication(field.name.to_owned()));
        }
        field_names.push(field.name.clone())
    }

    if key_count == 0 {
        return Err(GravityError::MissingKey(recd.name.clone()));
    }

    def.push(recd);
    Ok(())
}

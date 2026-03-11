use crate::{error::GravityError, parse::ast::Op};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::ast::{Expr, Program, Statement, Type};

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub name: String,
    pub expr: Expr,
    pub typ: Type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub vars: IndexMap<String, Value>,
    pub def: Vec<Assignment>,
    pub rel: IndexMap<String, Vec<Expr>>,
}

impl State {
    fn new() -> Self {
        Self {
            vars: IndexMap::<String, Value>::new(),
            def: Vec::<Assignment>::new(),
            rel: IndexMap::<String, Vec<Expr>>::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Number(i64),
    Decimal(f64),
    Text(String),
    Boolean(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::Decimal(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{:?}", v),
        }
    }
}

fn eval_expr(expr: &Expr, state: &State, name: &str) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Decimal(d) => Value::Decimal(*d),
        Expr::Bool(b) => Value::Boolean(*b),
        Expr::Text(s) => Value::Text(s.to_owned()),
        Expr::Ident(n) => state.vars.get(n).unwrap().clone(),
        Expr::SelfRef => state.vars.get(name).unwrap().clone(),
        Expr::Negate(e) => match eval_expr(e, state, name) {
            Value::Number(n) => Value::Number(-n),
            Value::Decimal(d) => Value::Decimal(-d),
            _ => unreachable!("how did this get past the type checker"),
        },
        Expr::Factorial(e, bangs) => {
            let val = eval_expr(e, state, name);
            match val {
                Value::Number(v) => {
                    let result = (1..=v)
                        .filter(|i| (v - i) % (*bangs as i64) == 0)
                        .fold(1i64, |acc, i| acc * i);
                    Value::Number(result)
                }
                _ => unreachable!(),
            }
        }
        Expr::BinOp(left, op, right) => {
            let lhs = eval_expr(left, state, name);
            let rhs = eval_expr(right, state, name);
            match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => match op {
                    Op::Add => Value::Number(l + r),
                    Op::Sub => Value::Number(l - r),
                    Op::Mul => Value::Number(l * r),
                    Op::Div => Value::Number(l / r),
                    Op::Mod => Value::Number(l.rem_euclid(r)),
                    Op::Pow => Value::Number(l.pow(r as u32)),
                },
                (Value::Decimal(l), Value::Decimal(r)) => match op {
                    Op::Add => Value::Decimal(l + r),
                    Op::Sub => Value::Decimal(l - r),
                    Op::Mul => Value::Decimal(l * r),
                    Op::Div => Value::Decimal(l / r),
                    Op::Mod => Value::Decimal(l.rem_euclid(r)),
                    Op::Pow => Value::Decimal(l.powf(r)),
                },
                (Value::Text(l), Value::Text(r)) => match op {
                    Op::Add => Value::Text(l + &r),
                    _ => unreachable!("Text can only be concatenated"),
                },
                _ => unreachable!(),
            }
        }
    }
}

pub fn eval_program(prg: Program) -> Result<State, GravityError> {
    let mut state = State::new();

    for stmt in prg.slf {
        match stmt {
            Statement::Assignment { name, expr, typ } => {
                state
                    .vars
                    .insert(name.clone(), eval_expr(&expr, &state, &name));
                state
                    .def
                    .push(Assignment { name: name.clone(), expr: expr, typ: typ })
            }
            Statement::Relationship { name, expr } => {
                state
                    .rel
                    .entry(name.clone())
                    .or_insert_with(|| Vec::new())
                    .push(expr.clone());

                state
                    .vars
                    .insert(name.clone(), eval_expr(&expr, &state, &name));
            }
        };
    }

    Ok(state)
}

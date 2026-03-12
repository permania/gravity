use std::{any::Any, default};

use crate::{error::GravityError, parse::ast::Op};
use indexmap::IndexMap;
use reedline_repl_rs::reedline::UndoBehavior;
use serde::{Deserialize, Serialize};

use super::ast::{Expr, Program, Statement, Type};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Assignment {
    pub typ: Type,
    pub name: String,
    pub expr: Expr,
}

impl From<Statement> for Assignment {
    fn from(stmt: Statement) -> Self {
        match stmt {
            Statement::Assignment { typ, name, expr } => Assignment { typ, name, expr },
            _ => panic!("expected Assignment"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub vars: IndexMap<String, Value>,
    pub def: Vec<Assignment>,
    pub rel: Vec<(String, Expr)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            vars: Default::default(),
            def: Default::default(),
            rel: Default::default(),
        }
    }
}

impl State {
    fn new() -> Self {
        Self {
            vars: IndexMap::<String, Value>::new(),
            def: Vec::<Assignment>::new(),
            rel: Vec::<(String, Expr)>::new(),
        }
    }

    pub fn compute(&mut self) -> () {
        self.vars.clear();
        for v in self.def.iter() {
            let base = eval_expr(&v.expr, self, &v.name);
            self.vars.insert(v.name.clone(), base);
        }
        for (name, expr) in self.rel.iter() {
            let val = eval_expr(expr, self, name);
            self.vars.insert(name.clone(), val);
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

impl Value {
    pub fn to_type(&self) -> Type {
        match self {
            Self::Number(_) => Type::Number,
            Self::Decimal(_) => Type::Decimal,
            Self::Text(_) => Type::Text,
            Self::Boolean(_) => Type::Bool,
        }
    }
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

pub fn eval_expr_nameless(expr: &Expr, state: &State) -> Result<Value, GravityError> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::Decimal(d) => Ok(Value::Decimal(*d)),
        Expr::Bool(b) => Ok(Value::Boolean(*b)),
        Expr::Text(s) => Ok(Value::Text(s.to_owned())),
        Expr::Ident(n) => Ok(state
            .vars
            .get(n)
            .ok_or(GravityError::UndefinedVariable(n.to_owned()))
            .cloned()?),
        Expr::Negate(e) => Ok(match eval_expr_nameless(e, state)? {
            Value::Number(n) => Value::Number(-n),
            Value::Decimal(d) => Value::Decimal(-d),
            _ => unreachable!("how did this get past the type checker"),
        }),
        Expr::Factorial(e, bangs) => Ok({
            let val = eval_expr_nameless(e, state)?;
            match val {
                Value::Number(v) => {
                    let result = (1..=v)
                        .filter(|i| (v - i) % (*bangs as i64) == 0)
                        .fold(1i64, |acc, i| acc * i);
                    Value::Number(result)
                }
                _ => unreachable!(),
            }
        }),
        Expr::BinOp(left, op, right) => {
            let lhs = eval_expr_nameless(left, state)?;
            let rhs = eval_expr_nameless(right, state)?;
            let lhs_t = lhs.to_type();
            let rhs_t = rhs.to_type();
            match (lhs, rhs) {
                (Value::Number(l), Value::Number(r)) => match op {
                    Op::Add => Ok(Value::Number(l + r)),
                    Op::Sub => Ok(Value::Number(l - r)),
                    Op::Mul => Ok(Value::Number(l * r)),
                    Op::Div => Ok(Value::Number(l / r)),
                    Op::Mod => Ok(Value::Number(l.rem_euclid(r))),
                    Op::Pow => Ok(Value::Number(l.pow(r as u32))),
                },
                (Value::Decimal(l), Value::Decimal(r)) => match op {
                    Op::Add => Ok(Value::Decimal(l + r)),
                    Op::Sub => Ok(Value::Decimal(l - r)),
                    Op::Mul => Ok(Value::Decimal(l * r)),
                    Op::Div => Ok(Value::Decimal(l / r)),
                    Op::Mod => Ok(Value::Decimal(l.rem_euclid(r))),
                    Op::Pow => Ok(Value::Decimal(l.powf(r))),
                },
                (Value::Text(l), Value::Text(r)) => match op {
                    Op::Add => Ok(Value::Text(l + &r)),
                    _ => unreachable!("Text can only be concatenated"),
                },
                _ => Err(GravityError::TypeMismatch(lhs_t, rhs_t)),
            }
        }
        Expr::SelfRef => Err(GravityError::SelfRef),
    }
}

fn eval_expr(expr: &Expr, state: &State, name: &str) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Decimal(d) => Value::Decimal(*d),
        Expr::Bool(b) => Value::Boolean(*b),
        Expr::Text(s) => Value::Text(s.to_owned()),
        Expr::Ident(n) => state.vars.get(n).unwrap().clone(),
        Expr::SelfRef => state.vars.get(name).cloned().unwrap_or_else(|| {
            state
                .def
                .iter()
                .find(|d| d.name == name)
                .map(|d| eval_expr(&d.expr, state, name))
                .unwrap()
        }),
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
                _ => unreachable!("type checker should block this"),
            }
        }
    }
}

pub fn eval_program(prg: Program) -> Result<State, GravityError> {
    let mut state = State::new();

    for stmt in prg.slf {
        match stmt {
            Statement::Assignment { name, expr, typ } => state.def.push(Assignment {
                name: name.clone(),
                expr: expr,
                typ: typ,
            }),
            Statement::Relationship { name, expr } => {
                state.rel.push((name.clone(), expr));
            }
        };
    }
    state.compute();

    Ok(state)
}

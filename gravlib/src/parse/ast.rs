use pest::{Parser, iterators::Pair};
use serde::{Serialize, Deserialize};
use pest_derive::Parser;

use crate::error::GravityError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct GravityParser;

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { typ: Type, name: String, expr: Expr },
    Relationship { name: String, expr: Expr },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(i64),
    Decimal(f64),
    Bool(bool),
    Text(String),
    Ident(String),
    SelfRef,
    Negate(Box<Expr>),
    Factorial(Box<Expr>, u32),
    BinOp(Box<Expr>, Op, Box<Expr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    Decimal,
    Bool,
    Text,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => write!(f, "Number"),
            Type::Decimal => write!(f, "Decimal"),
            Type::Bool => write!(f, "Boolean"),
            Type::Text => write!(f, "Text"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub slf: Vec<Statement>,
}

pub fn parse_program(contents: String) -> Program {
    let mut stmts = Vec::<Statement>::new();

    let pairs = GravityParser::parse(Rule::program, &contents).expect("parse failed");

    for pair in pairs {
        for statement in pair.into_inner() {
            if statement.as_rule() == Rule::statement {
                for inner in statement.into_inner() {
                    match inner.as_rule() {
                        Rule::assignment => {
                            let result = parse_assignment(inner);
                            stmts.push(result);
                        }
                        Rule::relationship => {
                            let result = parse_relationship(inner);
                            stmts.push(result);
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    Program { slf: stmts }
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::integer => Expr::Number(pair.as_str().parse().unwrap()),
        Rule::identifier => Expr::Ident(pair.as_str().to_string()),
        Rule::sum => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap());

            while let Some(op_pair) = inner.next() {
                let op = match op_pair.as_str() {
                    "+" => Op::Add,
                    "-" => Op::Sub,
                    _ => unreachable!(),
                };

                let rhs = parse_expr(inner.next().unwrap());
                lhs = Expr::BinOp(Box::<Expr>::new(lhs), op, Box::<Expr>::new(rhs));
            }
            lhs
        }
        Rule::product => {
            let mut inner = pair.into_inner();
            let mut left = parse_expr(inner.next().unwrap());

            while let Some(op_pair) = inner.next() {
                let op = match op_pair.as_str() {
                    "*" => Op::Mul,
                    "/" => Op::Div,
                    "&" => Op::Mod,
                    _ => unreachable!(),
                };
                let right = parse_expr(inner.next().unwrap());
                left = Expr::BinOp(Box::new(left), op, Box::new(right));
            }

            left
        }
        Rule::power => {
            let mut inner = pair.into_inner();
            let left = parse_expr(inner.next().unwrap());

            if let Some(right) = inner.next() {
                Expr::BinOp(Box::new(left), Op::Pow, Box::new(parse_expr(right)))
            } else {
                left
            }
        }
        Rule::bool_lit => Expr::Bool(pair.as_str() == "true"),
        Rule::decimal => Expr::Decimal(pair.as_str().parse().unwrap()),
        Rule::string => {
            let s = pair.as_str();
            Expr::Text(s[1..s.len() - 1].to_string())
        }
        Rule::unary => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Negate(Box::new(parse_expr(inner)))
        }
        Rule::factorial => {
            let mut inner = pair.into_inner();
            let atom = parse_expr(inner.next().unwrap());
            let bang_count = inner.count() as u32;
            Expr::Factorial(Box::new(atom), bang_count)
        }
        Rule::self_ref => Expr::SelfRef,
        _ => parse_expr(pair.into_inner().next().unwrap()),
    }
}

pub fn parse_assignment(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let vtype = match inner.next().unwrap().as_str() {
        "num" => Type::Number,
        "dec" => Type::Decimal,
        "bool" => Type::Bool,
        "text" => Type::Text,
        _ => unreachable!(),
    };
    let vname = inner.next().unwrap().as_str().to_string();
    let vvalue = parse_expr(inner.next().unwrap());

    Statement::Assignment {
        typ: vtype,
        name: vname,
        expr: vvalue,
    }
}

pub fn parse_relationship(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let vname = inner.next().unwrap().as_str().to_string();
    let vvalue = parse_expr(inner.next().unwrap());

    Statement::Relationship {
        name: vname,
        expr: vvalue,
    }
}

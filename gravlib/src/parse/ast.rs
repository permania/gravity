use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

use crate::error::GravityError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct GravityParser;

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment { typ: Type, name: String, expr: Expr },
    Relationship { name: String, expr: Expr },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Number,
    Decimal,
    Bool,
    Text,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => write!(f, "num"),
            Type::Decimal => write!(f, "dec"),
            Type::Bool => write!(f, "bool"),
            Type::Text => write!(f, "text"),
        }
    }
}

impl TryFrom<&str> for Type {
    type Error = GravityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "num" => Ok(Type::Number),
            "dec" => Ok(Type::Decimal),
            "bool" => Ok(Type::Bool),
            "text" => Ok(Type::Text),
            _ => Err(GravityError::InvalidString),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecField {
    pub typ: Type,
    pub name: String,
    pub is_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecDef {
    pub name: String,
    pub fields: Vec<RecField>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub slf: Vec<Statement>,
    pub rec: Vec<RecDef>,
}

pub fn parse_program(contents: String) -> Program {
    let mut stmts = Vec::<Statement>::new();
    let mut recs = Vec::<RecDef>::new();

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
                        Rule::rec_def => {
                            let result = parse_record(inner);
                            recs.push(result);
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    Program {
        slf: stmts,
        rec: recs,
    }
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
    let typ = match inner.next().unwrap().as_str() {
        "num" => Type::Number,
        "dec" => Type::Decimal,
        "bool" => Type::Bool,
        "text" => Type::Text,
        _ => unreachable!(),
    };
    let ident = inner.next().unwrap().as_str().to_string();
    let val = parse_expr(inner.next().unwrap());

    Statement::Assignment {
        typ,
        name: ident,
        expr: val,
    }
}

pub fn parse_relationship(pair: Pair<Rule>) -> Statement {
    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str().to_string();
    let val = parse_expr(inner.next().unwrap());

    Statement::Relationship {
        name: ident,
        expr: val,
    }
}

pub fn parse_record(pair: Pair<Rule>) -> RecDef {
    let mut fields: Vec<RecField> = Vec::new();

    let mut inner = pair.into_inner();
    let ident = inner.next().unwrap().as_str();

    for field in inner {
        for thing in field.into_inner() {
            let self_rule = thing.as_rule();
	    let is_key = self_rule == Rule::rec_key_field;
            let mut new_in = thing.into_inner();
            let typ = match new_in.next().unwrap().as_str() {
                "num" => Type::Number,
                "dec" => Type::Decimal,
                "bool" => Type::Bool,
                "text" => Type::Text,
                _ => unreachable!(),
            };
            let field_ident = new_in.next().unwrap().as_str();
            let name = field_ident.to_owned();
            fields.push(RecField { typ, name, is_key })
        }
    }

    RecDef {
        name: ident.to_owned(),
        fields,
    }
}

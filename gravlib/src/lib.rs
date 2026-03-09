use std::fs::File;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GravityParser;

const DB_EXT: &'static str = ".gravscm";

#[derive(Debug)]
pub enum Statement {
    Assignment { typ: Type, name: String, expr: Expr },
    Relationship { name: String, expr: Expr },
    Declaration { typ: Type, name: String },
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Choose,
}

#[derive(Debug)]
pub enum Type {
    Number,
    Decimal,
    Bool,
    Text,
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
        Rule::choose => {
            let mut inner = pair.into_inner();
            let left = parse_expr(inner.next().unwrap());

            if let Some(right) = inner.next() {
                Expr::BinOp(Box::new(left), Op::Choose, Box::new(parse_expr(right)))
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

pub fn initialize_db(name: String) -> Result<(), std::io::Error> {
    // let input = r#"dec b = 2.4; text a = "hi!";"#;
    let input = &std::fs::read_to_string(name + DB_EXT)?;

    let pairs = GravityParser::parse(Rule::program, input).expect("parse failed");

    for pair in pairs {
        for statement in pair.into_inner() {
            if statement.as_rule() == Rule::statement {
                for inner in statement.into_inner() {
		    match inner.as_rule() {
			Rule::assignment => {
			    let result = parse_assignment(inner);
			    println!("{:?}", result);
			}	
			Rule::relationship => {
			    let result = parse_relationship(inner);
			    println!("{:?}", result);
			}	
			_ => continue
		    }
                }
            }
        }
    }

    // for pair in pairs {
    //     // println!("{:#?}", pair.into_inner());
    // 	if pair.as_rule() == Rule::assignment {
    // 	    parse_assignment(pair);
    // 	}
    // }

    // let db = File::create_new(name + DB_EXT)?;
    // println!("{:?}", db);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}

use std::fs::File;

mod error;
use error::GravityError;

mod parse;
use parse::ast::{self, Rule};
use pest::Parser;

const DB_EXT: &str = ".gravdb";
const SCM_EXT: &str = ".gravscm";

pub fn read_db(name: String) -> Result<(), std::io::Error> {
    let input = &std::fs::read_to_string(name + SCM_EXT)?;
    let pairs = ast::GravityParser::parse(Rule::program, input).expect("parse failed");

    for pair in pairs {
        for statement in pair.into_inner() {
            if statement.as_rule() == Rule::statement {
                for inner in statement.into_inner() {
                    match inner.as_rule() {
                        Rule::assignment => {
                            let result = ast::parse_assignment(inner);
                            println!("{:?}", result);
                        }
                        Rule::relationship => {
                            let result = ast::parse_relationship(inner);
                            println!("{:?}", result);
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn initialize_db(name: String) -> Result<(), GravityError> {
    let db = File::create_new(name + DB_EXT)?;
    println!("{:?}", db);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

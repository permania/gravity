use std::collections::HashMap;

use gravlib::ast::{GravityParser, Type};
use gravlib::error::GravityError;
use gravlib::eval::eval_expr_nameless;
use gravlib::typecheck::expr_type_state;
use gravlib::{
    Assignment, GravityState, Rule, Value, ast, dump_db, dump_db_state, expr_to_string, typecheck,
};
use pest::Parser;
use reedline_repl_rs::Repl;
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::yansi::Paint;

fn print_state(
    _args: ArgMatches,
    context: &mut GravityState,
) -> Result<Option<String>, GravityError> {
    Ok(Some(format!("{:#?}", context)))
}

fn write_db(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let path = args.get_one::<String>("path").unwrap();
    dump_db_state(path.to_owned(), context)?;

    Ok(Some(format!(
        "Wrote {}",
        path.to_owned() + "." + gravlib::SCM_EXT
    )))
}

fn put(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let expr = args
        .get_many::<String>("expr")
        .unwrap()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!("num _output = {};", expr);
    let pair = GravityParser::parse(Rule::assignment, &content)?
        .next()
        .unwrap();
    let stmt = ast::parse_assignment(pair);

    let new_expr = if let ast::Statement::Assignment { expr, .. } = stmt {
        expr
    } else {
        unreachable!("not assignment: {:#?}", stmt)
    };

    let val = eval_expr_nameless(&new_expr, context)?;

    Ok(Some(format!("{}", val)))
}

fn add(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let typ = args.get_one::<String>("type").unwrap();
    let ident = args.get_one::<String>("ident").unwrap();
    let val_str = args.get_one::<String>("value").unwrap();
    let content = format!("{} {} = {};", typ, ident, val_str);
    let pair = GravityParser::parse(Rule::assignment, &content)?
        .next()
        .unwrap();
    let def = ast::parse_assignment(pair);

    let expr = if let ast::Statement::Assignment { expr, .. } = def.clone() {
        expr
    } else {
        unreachable!("assignment");
    };

    let mut defined: HashMap<String, Type> = context
        .def
        .iter()
        .map(|d| expr_type_state(&d.expr, context).map(|t| (d.name.clone(), t)))
        .collect::<Result<HashMap<_, _>, _>>()?;

    typecheck::check_assignment(&Type::try_from(typ.as_str())?, ident, &expr, &mut defined)?;

    if context.vars.contains_key(ident) {
        return Err(GravityError::Duplication(ident.to_owned()));
    }

    context.def.push(Into::<Assignment>::into(def));

    Ok(None)
}

fn remove(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let ident = args.get_one::<String>("var").unwrap();
    context.def.retain(|d| d.name != *ident);
    context.rel.shift_remove(ident);

    Ok(None)
}

fn relate(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let ident = args.get_one::<String>("var").unwrap();

    if !context.def.iter().any(|d| d.name == *ident) {
        return Err(GravityError::UndefinedVariable(ident.to_owned()));
    }

    let expr = args
        .get_many::<String>("expr")
        .unwrap()
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    let content = format!("{} <- {};", ident, expr);
    let pair = GravityParser::parse(Rule::relationship, &content)?
        .next()
        .unwrap();
    let rel = ast::parse_relationship(pair);

    let mut defined: HashMap<String, Type> = context
        .def
        .iter()
        .map(|d| expr_type_state(&d.expr, context).map(|t| (d.name.clone(), t)))
        .collect::<Result<HashMap<_, _>, _>>()?;

    let expr = if let ast::Statement::Relationship { expr, .. } = rel.clone() {
        expr
    } else {
        unreachable!("assignment");
    };

    typecheck::check_relationship(ident, &expr, &mut defined)?;

    context
        .rel
        .entry(ident.to_owned())
        .or_insert_with(Vec::new)
        .push(expr);

    Ok(None)
}

fn unrelate(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    todo!("derelate");
    Ok(None)
}

fn relates(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let ident = args.get_one::<String>("var").unwrap();

    if !context.def.iter().any(|d| d.name == *ident) {
        return Err(GravityError::UndefinedVariable(ident.to_owned()));
    }

    let var_def = context.rel.get(ident).cloned().map(|v| {
        v.iter()
            .map(|e| expr_to_string(e))
            .collect::<Vec<_>>()
            .join("\n")
    });

    Ok(var_def)
}

fn set(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let ident = args.get_one::<String>("var").unwrap();
    let val_str = args.get_one::<String>("value").unwrap();

    if !context.def.iter().any(|d| d.name == *ident) {
        return Err(GravityError::UndefinedVariable(ident.to_owned()));
    }

    let content = format!("num _d = {};", val_str);
    let pair = GravityParser::parse(Rule::assignment, &content)?
        .next()
        .unwrap();
    let stmt = ast::parse_assignment(pair);
    let new_expr = if let ast::Statement::Assignment { expr, .. } = stmt {
        expr
    } else {
        unreachable!("not assignment: {:#?}", stmt)
    };

    let var_def = context.def.iter().find(|d| d.name == *ident).unwrap();
    let expr_type = typecheck::expr_type_state(&new_expr, context)?;
    if expr_type != var_def.typ {
        return Err(GravityError::AssignmentMismatch(
            ident.to_owned(),
            var_def.typ.to_owned(),
            expr_type,
        ));
    }

    context
        .def
        .iter_mut()
        .find(|d| d.name == *ident)
        .ok_or(gravlib::error::GravityError::UndefinedVariable(
            ident.to_owned(),
        ))?
        .expr = new_expr;

    Ok(None)
}

pub fn run(name: String) -> Result<(), GravityError> {
    let state = gravlib::read_db_state(name).ok();

    let mut repl = Repl::new(state.or(Some(GravityState::default())).unwrap())
        .with_name("Gravity REPL")
        .with_version(&format!("{}", env!("CARGO_PKG_VERSION")))
        .with_command(
            Command::new("put")
                .about("Evaluate an expression and print the output")
                .arg(Arg::new("expr").required(true).num_args(1..)),
            put,
        )
        .with_command(
            Command::new("add")
                .about("Declare a new variable")
                .arg(
                    Arg::new("type")
                        .required(true)
                        .value_parser(["num", "dec", "text", "bool"]),
                )
                .arg(Arg::new("ident").required(true))
                .arg(Arg::new("value").required(true).allow_hyphen_values(true)),
            add,
        )
        .with_command(
            Command::new("remove")
                .about("Remove a variable from the database")
                .arg(Arg::new("var").required(true)),
            remove,
        )
        .with_command(
            Command::new("relate")
                .about("Declare a new relationship")
                .arg(Arg::new("var").required(true))
                .arg(
                    Arg::new("expr")
                        .required(true)
                        .num_args(1..)
                        .allow_hyphen_values(true),
                ),
            relate,
        )
        .with_command(
            Command::new("derel")
                .about("Remove an existing relationship")
                .arg(Arg::new("var").required(true))
                .arg(
                    Arg::new("expr")
                        .required(true)
                        .num_args(1..)
                        .allow_hyphen_values(true),
                ),
            unrelate,
        )
        .with_command(
            Command::new("rels")
                .about("Print a variable's relationships")
                .arg(Arg::new("var").required(true)),
            relates,
        )
        .with_command(
            Command::new("set")
                .about("Change the value of a variable")
                .arg(Arg::new("var").required(true))
                .arg(Arg::new("value").required(true).allow_hyphen_values(true)),
            set,
        )
        .with_command(
            Command::new("write")
                .about("Write the database schema to a file")
                .arg(Arg::new("path").required(true)),
            write_db,
        )
        .with_command(
            Command::new("state").about("DEBUG: print the database state"),
            print_state,
        )
        .without_clock()
        .with_prompt(&Paint::white("gravity ").to_string())
        .with_on_after_command(|ctx| {
            ctx.compute();
            Ok(None)
        });
    Ok(repl.run()?)
}

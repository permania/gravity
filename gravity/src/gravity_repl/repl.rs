use gravlib::ast::GravityParser;
use gravlib::error::GravityError;
use gravlib::{Assignment, GravityState, Rule, Value, ast};
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

fn put(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let var = args.get_one::<String>("var").unwrap();
    let val = context
        .vars
        .get(var)
        .ok_or(gravlib::error::GravityError::UndefinedVariable(
            var.to_owned(),
        ))?;
    Ok(Some(format!("{}", val.to_owned())))
}

fn add(args: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let typ = args.get_one::<String>("type").unwrap();
    let ident = args.get_one::<String>("ident").unwrap();
    let val_str = args.get_one::<String>("value").unwrap();
    let value = match typ.as_str() {
        "num" => val_str
            .parse::<i64>()
            .map(Value::Number)
            .map_err(|e| GravityError::ParseError(e.to_string()))?,
        "dec" => val_str
            .parse::<f64>()
            .map(Value::Decimal)
            .map_err(|e| GravityError::ParseError(e.to_string()))?,
        "bool" => match val_str.as_str() {
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            _ => Err(GravityError::ParseError(val_str.clone())),
        }?,
        "text" => Value::Text(val_str.clone()),
        _ => unreachable!(),
    };
    let content = format!("{} {} = {};", typ, ident, value);
    let program = ast::parse_program(content);
    let stmt = program.slf.into_iter().next().unwrap();

    if context.vars.contains_key(ident) {
        return Err(GravityError::Duplication(ident.to_owned()));
    }
    context.def.push(Assignment::from(stmt));

    Ok(None)
}

fn remove(arg: ArgMatches, context: &mut GravityState) -> Result<Option<String>, GravityError> {
    let ident = arg.get_one::<String>("var").unwrap();
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

    if let ast::Statement::Relationship { expr, .. } = rel {
        context
            .rel
            .entry(ident.to_owned())
            .or_insert_with(Vec::new)
            .push(expr)
    } else {
        unreachable!();
    }

    Ok(None)
}

pub fn run(name: String) -> Result<(), GravityError> {
    let state = gravlib::read_db_state(name).ok();

    let mut repl = Repl::new(state.or(Some(GravityState::default())).unwrap())
        .with_name("Gravity REPL")
        .with_version(&format!("{}", env!("CARGO_PKG_VERSION")))
        .with_command(
            Command::new("put")
                .about("Print the current value of a variable")
                .arg(Arg::new("var").required(true)),
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
                .arg(Arg::new("value").required(true)),
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

use gravlib::GravityState;
use gravlib::error::GravityError;
use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::yansi::Paint;
use reedline_repl_rs::{Repl};

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

pub fn run(name: String) -> Result<(), GravityError> {
    let state = gravlib::read_db_state(name).ok();

    let mut repl = Repl::new(state.or(Some(GravityState::default())).unwrap())
        .with_name("Gravity REPL")
        .with_version(&format!("{}", env!("CARGO_PKG_VERSION")))
        .with_command(
            Command::new("put")
                .about("Print the current state of the database")
                .arg(Arg::new("var").required(true)),
            put,
        )
        .without_clock()
        .with_prompt(&Paint::white("gravity ").to_string());
    Ok(repl.run()?)
}

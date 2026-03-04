use std::{
  env,
  io::{self},
  path::Path,
};

mod commands;
use cardbox::imp_std::common::eprint;
use tap::Pipe;

#[cfg(feature = "copy-all")]
use crate::copy::copy_all;

fn main() -> io::Result<()> {
  let args = env::args().collect::<Vec<_>>();

  let rest_args = args
    .get(1..)
    // &[] => None
    .filter(|x| !x.is_empty());

  Path::new(&args[0])
    .file_stem()
    .and_then(|x| x.to_str())
    .expect("program file stem is invalid")
    .pipe(|x| parse_and_run_command(x, rest_args))
}

fn run_cardbox(rest_args: Option<&[String]>) -> io::Result<()> {
  let (first, rest) = match rest_args.and_then(|a| a.split_first()) {
    Some((first, rest)) => (first, rest),
    _ => return commands::help_info(),
  };
  // eprintln!("first: {first}, rest: {rest:?}");

  match first.as_ref() {
    "-v" | "--version" => commands::display_version(),
    "--list" => commands::list_all_commands(),
    #[cfg(feature = "serde")]
    "--list-json" => commands::list_all_commands_json(),
    "-h" | "--help" | "" => commands::help_info(),
    x if commands::all_available_commands().contains(&x) => {
      let rest_args = if rest.is_empty() { None } else { Some(rest) };
      parse_and_run_command(x, rest_args)
    }
    _ => {
      eprint("[WARN] Unknown command: ")?;
      eprint(first)?;
      commands::help_info()
    }
  }
}
// ===================
#[cfg(any(feature = "copy-file", feature = "copy-all"))]
mod copy;

// ===================
fn parse_and_run_command(
  program_stem: &str,
  rest_args: Option<&[String]>,
) -> io::Result<()> {
  match program_stem {
    #[cfg(feature = "copy-file")]
    "copy-file" => copy::copy_file(rest_args),
    //
    #[cfg(feature = "copy-all")]
    "copy-all" => copy_all(rest_args),
    //
    #[cfg(feature = "target")]
    "target" => {
      todo!()
    }
    //
    #[cfg(feature = "uts-name")]
    "uts-name" => {
      todo!()
    }
    //
    #[cfg(feature = "list")]
    "list" => {
      todo!()
    }
    //
    #[cfg(feature = "rename")]
    "rename" => {
      todo!()
    }
    //
    #[cfg(feature = "run-command")]
    "run-command" => {
      todo!()
    }
    _ if rest_args.is_none() => commands::help_info(),
    _ => run_cardbox(rest_args),
  }
}

use std::{
  env,
  io::{self},
  path::Path,
};

mod commands;
use cardbox::utils::eprint;
use tap::Pipe;

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
    #[cfg(feature = "serde")]
    "--version" => commands::display_version_in_json(),

    "-V" => commands::display_version(),
    "-L" => commands::list_all_commands(),

    #[cfg(feature = "serde")]
    "--list" => commands::list_all_commands_in_json(),
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

#[cfg(feature = "target")]
mod target_info;

#[cfg(feature = "list")]
mod list;

// ===================
fn parse_and_run_command(
  program_stem: &str,
  rest_args: Option<&[String]>,
) -> io::Result<()> {
  match program_stem {
    #[cfg(feature = "copy-file")]
    "copy-file" => copy::copy_file::run(rest_args),
    //
    #[cfg(feature = "copy-all")]
    "copy-all" => copy::copy_all::run(rest_args),
    // target + uts-name
    #[cfg(feature = "target")]
    "target" => target_info::run(rest_args),
    //
    #[cfg(feature = "list")]
    "list" => list::run(rest_args),
    //
    #[cfg(feature = "rename")]
    "rename" => {
      todo!()
    }
    //
    #[cfg(feature = "run-cmd")]
    "run-cmd" => {
      todo!()
    }
    _ if rest_args.is_none() => commands::help_info(),
    _ => run_cardbox(rest_args),
  }
}

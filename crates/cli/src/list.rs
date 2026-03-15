use std::io;

use cardbox::utils::puts;
use tap::Pipe;

use crate::commands::contains_help;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_list_help as help;

  // args is_empty() or None => help()
  let args = match args {
    Some(&[]) => return help(),
    Some(x) => x,
    _ => return help(),
  };
  if contains_help(args) {
    return help();
  }

  Ok(())
}

fn display_list_help() -> io::Result<()> {
  r#"
Usage:
  list [/path/to/file] [/path/to/dir]

Examples:
  list .
  list /tmp
  list /tmp/a.txt
  list a.txt b.txt
  "#
  .pipe(puts)?;
  Ok(())
}

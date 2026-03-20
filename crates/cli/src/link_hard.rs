use std::{
  io::{self},
  path::Path,
};

use cardbox::{copy::error::io_invalid_input, utils::puts};
use tap::Pipe;

use crate::commands::is_first_help_flag;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  let paths = match args {
    Some(&[]) | None => return help(),
    Some(x) => x,
  };

  if is_first_help_flag(paths) {
    return help();
  }

  if paths.len() < 2 {
    err_not_enough_args()?
  }
  let [src_path, dst_path] = [&paths[0], &paths[1]].map(Path::new);

  cardbox::fs::link::link_hard(src_path, dst_path)
}

fn err_not_enough_args() -> Result<(), io::Error> {
  "[ERR] Not enough arguments
    Usage: link-hard [/path/to/src-file] [/path/to/dst]"
    .pipe(io_invalid_input)
    .pipe(Err)
}

fn help() -> io::Result<()> {
  r#"
Usage:
  link-hard [/path/to/src-file] [/path/to/dst-file]

Examples:
  link-hard ./tmp/a.txt .
  link-hard a.txt b.txt
  "#
  .pipe(puts)?;
  Ok(())
}

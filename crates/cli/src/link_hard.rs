use std::{
  io::{self},
  path::Path,
};

use cardbox::{
  copy::{
    error::io_invalid_input,
    file::{create_dst_parent_dir, validate_and_resolve_dst_path},
  },
  utils::{eputs, puts},
};
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
    "[ERR] Not enough arguments
    Usage: link-hard [/path/to/src-file] [/path/to/dst]"
      .pipe(io_invalid_input)
      .pipe(Err)?
  }
  let [src_path, dst_path] = [&paths[0], &paths[1]].map(Path::new);
  create_dst_parent_dir(dst_path)?;

  let dst_path = validate_and_resolve_dst_path(src_path, dst_path)?;

  if dst_path.exists()
    && let Err(e) = std::fs::remove_file(&dst_path)
  {
    eputs("[WARN] Failed to remove existing file")?;
    eputs(e.to_string())?;
  }

  std::fs::hard_link(src_path, dst_path)
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

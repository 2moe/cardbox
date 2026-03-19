#![cfg_attr(target_os = "wasi", feature(wasi_ext))]

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
    Usage: link-sym [/path/to/src] [/path/to/dst]"
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

  // eprintln!("symlink: {dst_path:?} -> {src_path:?}");

  #[cfg(unix)]
  std::os::unix::fs::symlink(src_path, dst_path)?;

  #[cfg(windows)]
  {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    match src_path {
      p if p.is_dir() => symlink_dir(src_path, dst_path),
      _ => symlink_file(src_path, dst_path),
    }
  }?;

  #[cfg(target_os = "wasi")]
  std::os::wasi::fs::symlink_path(src_path, dst_path)?;

  Ok(())
}

fn help() -> io::Result<()> {
  r#"
Usage:
  link-sym [/path/to/src] [/path/to/dst]

Examples:
  link-sym /tmp/ .
  link-sym /tmp/a.txt .
  link-sym a.txt b.txt
  "#
  .pipe(puts)?;
  Ok(())
}

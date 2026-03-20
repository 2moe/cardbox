use std::{
  io::{self},
  path::Path,
};

use cardbox::{
  copy::{
    error::{io_invalid_input, reject_non_dir_dst_for_multi_files},
    file::create_dst_parent_dir,
  },
  fs::rename_path,
  path::split_last_path,
  utils::puts,
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
    err_not_enough_args()?
  }

  // ===
  let (dst_path, src_strs) = split_last_path(paths);

  create_dst_parent_dir(dst_path)?;

  if src_strs.len() == 1 {
    let src_path = Path::new(&src_strs[0]);
    return rename_path(src_path, dst_path);
  }

  // === args.len() >= 3 ===
  // file1 file2 dst_dir
  // 当参数个数 >=3 时，dst_path 必须是目录。
  reject_non_dir_dst_for_multi_files(dst_path)?;
  std::fs::create_dir_all(dst_path)?;

  rename_all_files_to_dir(src_strs, dst_path)?;
  // ===
  Ok(())
}

fn err_not_enough_args() -> Result<(), io::Error> {
  "[ERR] Not enough arguments
    Usage: rename [/path/to/old] [/path/to/new]"
    .pipe(io_invalid_input)
    .pipe(Err)
}

fn rename_all_files_to_dir(src_strs: &[String], dst_path: &Path) -> io::Result<()> {
  for src in src_strs.iter().map(Path::new) {
    rename_path(src, dst_path)?
  }
  Ok(())
}

fn help() -> io::Result<()> {
  r#"
Usage:
  rename [/path/to/old.1] [/path/to/old.2] [/path/to/new]

Examples:
  rename ./tmp/a.txt .
  rename a.txt b.txt
  "#
  .pipe(puts)?;
  Ok(())
}

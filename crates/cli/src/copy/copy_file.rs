use std::{fs, io, path::Path};

use cardbox::imp_std::{
  common::{eprint, eputs},
  copy::copy_file::{
    copy_from_stdin_to_file, copy_src_to_dst_file, resolve_dst_file_path,
  },
};
use tap::Pipe;

use crate::commands::contains_help;

#[cfg(feature = "copy-file")]
pub(crate) fn copy_file(args: Option<&[String]>) -> io::Result<()> {
  use cardbox::imp_std::copy::copy_file::create_dst_dir;
  use display_copy_file_help as help;

  // args.is_empty()
  let Some(args) = args else {
    return help();
  };
  if contains_help(args) {
    return help();
  }

  let (dst_path, src_paths) = args
    .split_last()
    .expect("Failed to get destination path")
    .pipe(|(d, s)| (Path::new(d), s));

  create_dst_dir(dst_path)?;

  if src_paths.is_empty() {
    return copy_from_stdin_to_file(dst_path);
  }
  if src_paths.len() == 1 {
    return copy_src_to_dst_file(&src_paths[0], dst_path);
  }

  // args.len() >= 3
  if dst_path.exists() && !dst_path.is_dir() {
    use cardbox::imp_std::copy::copy_file::io_invalid_input;

    "Destination path is not a directory.
    Please provide a valid directory path."
      .pipe(io_invalid_input)
      .pipe(Err)?;
  }
  fs::create_dir_all(dst_path)?;

  for src in src_paths.iter().map(Path::new) {
    if src.is_dir() {
      eprint("Skipping directory: ")?;
      eputs(src.to_string_lossy().as_bytes())?;
      continue;
    }

    let Some(cow_dst_path) = resolve_dst_file_path(dst_path, src) else {
      continue;
    };

    fs::copy(src, cow_dst_path)?;
  }

  Ok(())
}

#[cfg(feature = "copy-file")]
pub(crate) fn display_copy_file_help() -> io::Result<()> {
  use cardbox::imp_std::common::puts;

  "
Usage:
  copy-file [/path/to/src_file] [/path/to/dst_file]
  OR: copy-file [/path/to/dst_file]
  OR: copy-file [file_1] [file_2] [file_n..] [/path/to/dir]

e.g.,
  copy-file /tmp/a.txt b.txt

  // copy file from stdin to tmp.txt
  copy-file tmp.txt

  // copy multiple files to a directory
  copy-file a1.txt a2.txt a3.txt /tmp
  "
  .pipe(puts)?;

  Ok(())
}

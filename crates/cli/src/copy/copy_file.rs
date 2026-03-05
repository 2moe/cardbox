use std::{fs, io, path::Path};

use cardbox::imp_std::{
  common::{eprint, eputs},
  copy::file::{
    copy_from_stdin_to_file, copy_src_to_dst_file, create_dst_dir, io_invalid_input,
  },
};
use tap::Pipe;

use crate::{
  commands::contains_help,
  copy::{eputs_path, split_last_path},
};

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_copy_file_help as help;

  // args.is_empty()
  let Some(args) = args else {
    return help();
  };
  if contains_help(args) {
    return help();
  }

  let (dst_path, src_strs) = split_last_path(args);

  create_dst_dir(dst_path)?;

  if src_strs.is_empty() {
    return copy_from_stdin_to_file(dst_path);
  }
  if src_strs.len() == 1 {
    return copy_src_to_dst_file(&src_strs[0], dst_path);
  }

  if dst_path.exists() && !dst_path.is_dir() {
    r#"
    args.len() >= 3;
    Destination path is not a directory.

    Sorry! This function does not support the concatenation of
    multiple files into a single file.

    Instead, it supports copying multiple files to a directory.
    Please provide a valid directory path."#
      .pipe(io_invalid_input)
      .pipe(Err)?;
  }
  fs::create_dir_all(dst_path)?;

  for src in src_strs.iter().map(Path::new) {
    if src.is_dir() {
      eprint("[WARN] src is a directory; Skipping: ")?;
      eputs_path(src)?;
      continue;
    }

    // ignore Error
    if let Err(e) = copy_src_to_dst_file(src, dst_path) {
      eprint("[WARN] Skipping invalid destination file: ")?;
      eputs_path(dst_path)?;
      eputs(e.to_string())?;
      continue;
    }
  }
  Ok(())
}

pub(crate) fn display_copy_file_help() -> io::Result<()> {
  use cardbox::imp_std::common::puts;

  r##"
Usage:
      copy-file [/path/to/src_file] [/path/to/dst_file]
  OR: copy-file [/path/to/dst_file]
  OR: copy-file [file_1] [file_2] [file_n..] [/path/to/dir]

Note: "-" is stdin, use "-" as src to copy from stdin to dst_file.

e.g.,
  copy-file /tmp/a.txt b.txt

  // copy file from stdin to tmp.txt
  copy-file tmp.txt

  // copy multiple files to a directory
  copy-file a1.txt a2.txt a3.txt /tmp
  "##
    .pipe(puts)?;

  Ok(())
}

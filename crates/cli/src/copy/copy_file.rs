use std::{fs, io, path::Path};

use cardbox::imp_std::{
  common::{eprint, eputs},
  copy::{
    error::reject_non_dir_dst_for_multi_files,
    file::{copy_from_stdin_to_file, copy_src_to_dst_file, create_dst_dir},
  },
  path::{eputs_path, split_last_path},
};
use tap::Pipe;

use crate::commands::contains_help;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_copy_file_help as help;

  // args is_empty() or None => help()
  let args = match args {
    Some(&[]) => return help(),
    Some(x) => x,
    _ => return help(),
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
    return copy_src_to_dst_file(&src_strs[0], dst_path, true);
  }

  // === args.len() >= 3 ===
  reject_non_dir_dst_for_multi_files(dst_path)?;
  fs::create_dir_all(dst_path)?;
  copy_all_files_to_dir(src_strs, dst_path)?;
  Ok(())
}

fn copy_all_files_to_dir(src_strs: &[String], dst_path: &Path) -> io::Result<()> {
  // Allow one occurrence of stdin, but not two or more.
  let mut stdin_found = false;

  for src in src_strs.iter().map(Path::new) {
    if src.is_dir() {
      eprint("[WARN] src is a directory; Skipping: ")?;
      eputs_path(src)?;
      continue;
    }

    let res = match stdin_found {
      false if src == "-" => {
        stdin_found = true;
        copy_from_stdin_to_file(dst_path)
      }
      _ => copy_src_to_dst_file(src, dst_path, false),
    };

    // ignore Error
    if let Err(e) = res {
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

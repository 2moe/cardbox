use std::{fs, io, path::Path};

use cardbox::imp_std::copy::{
  extra::{copy_all as fs_copy_all, cp_file_options, fs_extra},
  file::{copy_from_stdin_to_file, copy_src_to_dst_file, create_dst_dir},
};
use tap::Pipe;

use crate::{commands::contains_help, copy::split_last_path};

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_copy_all_help as help;

  let Some(args) = args else {
    return help();
  };
  if contains_help(args) {
    return help();
  }

  // cardbox::imp_std::copy::copy_all(from, to, overwrite);
  let (dst_path, src_strs) = split_last_path(args);

  create_dst_dir(dst_path)?;

  if src_strs.is_empty() {
    return copy_from_stdin_to_file(dst_path);
  }

  if src_strs.len() == 1 {
    let src = &src_strs[0];
    let src_path = Path::new(src);
    if !src_path.is_dir() {
      return copy_src_to_dst_file(src, dst_path);
    }
    fs_copy_all(src_path, dst_path).map_err(io::Error::other)?;
    return Ok(());
  }

  // === args.len() >= 3 ===

  // zh: 允许出现一次 stdin， 不允许出现两次。
  // en: Allow one occurrence of stdin, but not two or more.
  let mut stdin_found = false;
  for src in src_strs.iter().map(Path::new) {
    if src.is_dir() {
      fs_copy_all(src, dst_path).map_err(io::Error::other)?;
      continue;
    }
    // if src is not a dir:
    fs::create_dir_all(dst_path)?;
    match stdin_found {
      false if src == "-" => {
        stdin_found = true;
        copy_from_stdin_to_file(dst_path)?;
      }
      _ => {
        fs_extra::file::copy(src, dst_path, &cp_file_options())
          .map_err(io::Error::other)?;
      }
    }
  }

  Ok(())
}

pub(crate) fn display_copy_all_help() -> io::Result<()> {
  use cardbox::imp_std::common::puts;

  r##"
Usage:
      copy-all [/path/to/src_file] [/path/to/dst_file]
  OR: copy-all [/path/to/src_dir] [/path/to/dst_dir]
  OR: copy-all [/path/to/dst_file]
  OR: copy-all [file_1] [file_2] [file_n..] [/path/to/dir]
  OR: copy-all [dir_1] [dir_2] [dir_n..] [/path/to/dir]

Note: "-" is stdin, use "-" as src to copy from stdin to dst_path.

e.g.,
  copy-all /tmp/dir_a dir_b

  // copy file from stdin to tmp.txt
  copy-all tmp.txt

  // copy multiple files to a directory
  copy-all a1.txt a2.txt a3.txt /tmp

  // copy multiple dirs to a directory
  copy-all dir_a dir_b dir_c /tmp
  "##
    .pipe(puts)?;

  Ok(())
}

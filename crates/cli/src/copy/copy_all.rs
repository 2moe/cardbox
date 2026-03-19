use std::{fs, io, path::Path};

use cardbox::{
  copy::{
    error::reject_non_dir_dst_for_multi_files,
    extra::copy_all as fs_copy_all,
    file::{copy_from_stdin_to_file, copy_src_to_dst_file, create_dst_dir},
  },
  path::split_last_path,
  utils::puts,
};
use tap::Pipe;

use crate::commands::is_first_help_flag;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_copy_all_help as help;

  // args is_empty() or None => help()
  let args = match args {
    Some(&[]) | None => return help(),
    Some(x) => x,
  };
  if is_first_help_flag(args) {
    return help();
  }

  let (dst_path, src_strs) = split_last_path(args);

  create_dst_dir(dst_path)?;

  if src_strs.is_empty() {
    return copy_from_stdin_to_file(dst_path);
  }

  if src_strs.len() == 1 {
    return match Path::new(&src_strs[0]) {
      src if src.is_dir() => fs_copy_all(src, dst_path)
        .map_err(io::Error::other)
        .map(|_| ()),
      src => copy_src_to_dst_file(src, dst_path, true),
    };
  }

  // === args.len() >= 3 ===
  reject_non_dir_dst_for_multi_files(dst_path)?;
  copy_multi_paths_to_dst(src_strs, dst_path)?;
  Ok(())
}

fn copy_multi_paths_to_dst(src_strs: &[String], dst_path: &Path) -> io::Result<()> {
  // zh: 允许出现一次 stdin， 不允许出现两次。
  // en: Allow one occurrence of stdin, but not two or more.
  let mut stdin_found = false;

  for src in src_strs.iter().map(Path::new) {
    if src.is_dir() {
      fs_copy_all(src, dst_path).map_err(io::Error::other)?;
      continue;
    }

    // if src is not a dir => copy_stdin or copy_file or copy_all
    fs::create_dir_all(dst_path)?;
    match stdin_found {
      false if src == "-" => {
        stdin_found = true;
        copy_from_stdin_to_file(dst_path)
      }
      _ => copy_src_to_dst_file(src, dst_path, false),
    }?
  }
  Ok(())
}

pub(crate) fn display_copy_all_help() -> io::Result<()> {
  r##"
Usage:
      copy-all [/path/to/src_file] [/path/to/dst_file]
  OR: copy-all [/path/to/src_dir] [/path/to/dst_dir]
  OR: copy-all [/path/to/dst_file]
  OR: copy-all [file_1] [file_2] [file_n..] [/path/to/dir]
  OR: copy-all [dir_1] [dir_2] [dir_n..] [/path/to/dir]

Note: "-" is stdin, use "-" as src_file to copy from stdin to dst_path.

e.g.,
  copy-all /tmp/dir_a dir_b

  // - en: When reading from standard input, press **Ctrl+D** to indicate
  //       end-of-file (EOF) after completing the input.
  // - zh: 当从 stdin 读取数据时，您可以在输入完成后，按下 Ctrl+D 来退出。
  //
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

use std::{fs, io, path::Path};

use cardbox::{
  copy::{
    error::reject_non_dir_dst_for_multi_files,
    file::{copy_from_stdin_to_file, copy_src_to_dst_file, create_dst_parent_dir},
  },
  path::{eputs_path, split_last_path},
  utils::{eprint, eputs, puts},
};
use tap::Pipe;

use crate::commands::is_first_help_flag;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  use display_copy_file_help as help;

  // args is_empty() or None => help()
  let paths = match args {
    Some(&[]) | None => return help(),
    Some(x) => x,
  };

  if is_first_help_flag(paths) {
    return help();
  }

  let (dst_path, src_strs) = split_last_path(paths);

  create_dst_parent_dir(dst_path)?;

  if src_strs.is_empty() {
    return copy_from_stdin_to_file(dst_path);
  }
  if src_strs.len() == 1 {
    return copy_src_to_dst_file(&src_strs[0], dst_path, true);
  }

  // === args.len() >= 3 ===
  // file1 file2 dst_dir
  // 当参数个数 >=3 时，dst_path 必须是目录。
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
  r##"
Usage:
      copy-file [/path/to/src_file] [/path/to/dst_file]
  OR: copy-file [/path/to/dst_file]
  OR: copy-file [file_1] [file_2] [file_n..] [/path/to/dir]

Note: "-" is stdin, use "-" as src_file to copy from stdin to dst_file.

e.g.,
  // copy file from /tmp/a.txt to b.txt
  copy-file /tmp/a.txt b.txt

  // copy file from a.txt to ./tmp/a.txt
  copy-file a.txt tmp/

  // - en:
  //   - POSIX-sh => ^D => EOF
  //   - Windows-CMD => ^Z, Enter => EOF
  //   When reading from standard input (stdin),
  //   you can press Ctrl+D after completing the input to signal EOF
  //   and terminate the input.
  // - zh: 当从 stdin 读取数据时，您可以在输入完成后，按下 Ctrl+D 来退出。
  //
  // copy file from stdin to tmp.txt
  copy-file tmp.txt

  // copy file from stdin to tmp.txt
  copy-file - tmp.txt

  // copy "./-" (non-stdin) to tmp.txt
  copy-file ./- tmp.txt

  // copy file from stdin to ./tmp/-
  copy-file tmp/

  // copy multiple files to a directory
  copy-file a1.txt a2.txt a3.txt /tmp
  "##
    .pipe(puts)?;

  Ok(())
}

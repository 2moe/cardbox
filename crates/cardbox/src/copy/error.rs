use std::{io, path::Path};

use tap::Pipe;

use crate::{path::eputs_path, utils::eprint};

/// This function checks whether the last command-line argument (`dst_path`) is
/// a directory and returns an `io::Result` carrying an error message if it is
/// not.
///
/// > NOTE: If multiple files are passed to a non-directory path in the CLI, an
/// > error should be returned.
/// >
/// > e.g.,
/// > - bad: `copy-file f1.txt f2.txt d1.txt`
/// > - good: `copy-file f1.txt f2.txt dir_1/`
///
/// <details>
///  <summary>zh</summary>
/// 此函数主要用于判断命令行参数的最后一个参数是否为目录，并返回包含错误消息的
/// io::Result。
///
/// > 注： 若在 CLI 中传递多个文件到一个非目录路径时，则应该报错。
/// </details>
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use cardbox::imp_std::copy::error::reject_non_dir_dst_for_multi_files;
///
/// // If `/tmp/out.txt` already exists and is a plain file:
/// assert!(reject_non_dir_dst_for_multi_files(Path::new("/tmp/out.txt")).is_err());
/// ```
pub fn reject_non_dir_dst_for_multi_files(dst_path: &Path) -> io::Result<()> {
  let err_msg = r#"
    Destination path is not a directory.

    Sorry! This function does not support the concatenation of
    multiple files into a single file.

    Instead, it supports copying multiple files to a directory.
    Please provide a valid directory path."#;

  if dst_path.exists() && !dst_path.is_dir() {
    eprint("[ERROR] Not a directory: ")?;
    eputs_path(dst_path)?;
    io_not_a_dir(err_msg).pipe(Err)?
  }
  Ok(())
}

pub fn io_not_a_dir(s: &str) -> io::Error {
  io::Error::new(io::ErrorKind::NotADirectory, s)
}

pub fn io_invalid_input(s: &str) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, s)
}

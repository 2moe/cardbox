use std::{borrow::Cow, fs, io, path::Path};

use rustix::stdio;
use tap::Pipe;

use crate::imp_std::fs::create_a_new_buf_writer;

/// Ensure the destination directory exists.
///
/// - If `dst_path` is a file, create its parent directories.
/// - If `dst_path` is a non-existing directory, create it recursively.
/// - If the directory already exists, do nothing!
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use cardbox::imp_std::copy::copy_file::create_dst_dir;
///
/// create_dst_dir(Path::new("/tmp/deep/nested/file.txt"))?; // creates `/tmp/deep/nested`
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn create_dst_dir(dst_path: &Path) -> io::Result<()> {
  match dst_path.is_dir() {
    false => dst_path
      .parent()
      .filter(|p| !p.exists())
      .map(fs::create_dir_all)
      .transpose()
      .map(|_o| ()),
    _ if !dst_path.exists() => fs::create_dir_all(dst_path),
    _ => Ok(()),
  }
}

/// Copy everything from standard-input into the given file.
///
/// If `dst_path` is a directory the data is written to a file named `-` inside
/// that directory; otherwise the file itself is overwritten.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use cardbox::imp_std::copy::copy_file::copy_from_stdin_to_file;
///
/// copy_from_stdin_to_file(Path::new("/tmp/out.txt"))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn copy_from_stdin_to_file(dst_path: &Path) -> io::Result<()> {
  let mut lock = io::stdin().lock();

  match dst_path.is_dir() {
    true => dst_path
      .join("-")
      .pipe(Cow::from),
    _ => dst_path.into(),
  }
  .pipe(create_a_new_buf_writer)?
  .pipe_ref_mut(|dst| io::copy(&mut lock, dst))?;

  Ok(())
}

pub fn io_invalid_input(s: &str) -> io::Error {
  io::Error::new(io::ErrorKind::InvalidInput, s)
}

/// Copy a single file or stdin to the requested destination.
///
/// - If `src` is `"-"`, data are read from **stdin** (see
/// - [`copy_from_stdin_to_file`]).
/// - If `dst_path` is a **directory**, the source file name is appended to it.
/// - If source and destination are identical, an error is returned immediately.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use cardbox::imp_std::copy::copy_file::copy_src_to_dst_file;
///
/// // Copy file into another name
/// copy_src_to_dst_file("Cargo.toml", Path::new("/tmp/x.toml"))?;
///
/// // Copy file into existing directory
/// copy_src_to_dst_file("Cargo.toml", Path::new("/tmp"))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn copy_src_to_dst_file<S: AsRef<Path>, D: AsRef<Path>>(
  src: S,
  dst: D,
  // is_src_stdin: bool,
) -> io::Result<()> {
  let (src_path, dst_path) = (src.as_ref(), dst.as_ref());
  if src_path.eq("-") {
    return copy_from_stdin_to_file(dst_path);
  }

  if src_path.is_dir() {
    "src is a directory, use `copy-all` instead of `copy-file`!"
      .pipe(io_invalid_input)
      .pipe(Err)?;
  }

  if src_path == dst_path {
    "src and dst are the same file, nothing to do!"
      .pipe(io_invalid_input)
      .pipe(Err)?;
  }

  let dst_path = resolve_dst_file_path(dst_path, src_path)
    .ok_or_else(|| io_invalid_input("Destination path should not be a directory"))?;

  fs::copy(src_path, dst_path)?;
  Ok(())
}

pub fn resolve_dst_file_path<'a>(
  dst_path: &'a Path,
  src_path: &'a Path,
) -> Option<Cow<'a, Path>> {
  match dst_path.is_dir() {
    true => src_path
      .file_name()
      .map(|x| dst_path.join(x))
      .map(Cow::from),
    _ => Some(dst_path.into()),
  }
}

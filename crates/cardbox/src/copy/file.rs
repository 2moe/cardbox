use std::{borrow::Cow, fs, io, path::Path};

use tap::Pipe;

use crate::{copy::error::io_invalid_input, fs::create_a_new_buf_writer};

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
pub fn create_dst_parent_dir<P: AsRef<Path>>(dst: P) -> io::Result<()> {
  let dst_path = dst.as_ref();

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
/// copy_src_to_dst_file("Cargo.toml", Path::new("/tmp/x.toml"), false)?;
///
/// // Copy file into existing directory
/// copy_src_to_dst_file("Cargo.toml", Path::new("/tmp"), false)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn copy_src_to_dst_file<S: AsRef<Path>, D: AsRef<Path>>(
  src: S,
  dst: D,
  check_stdin_in_src: bool,
) -> io::Result<()> {
  let (src_path, dst_path) = (src.as_ref(), dst.as_ref());

  if check_stdin_in_src && src_path == "-" {
    return copy_from_stdin_to_file(dst_path);
  }

  if src_path.is_dir() {
    "src is a directory, use `copy-all` instead of `copy-file`!"
      .pipe(io_invalid_input)
      .pipe(Err)?;
  }

  let dst_path = validate_and_resolve_dst_path(src_path, dst_path)?;

  fs::copy(src_path, dst_path)?;
  Ok(())
}

pub fn validate_and_resolve_dst_path<'a>(
  src_path: &'a Path,
  dst_path: &'a Path,
) -> io::Result<Cow<'a, Path>> {
  if src_path == dst_path {
    "src and dst are the same path, nothing to do!"
      .pipe(io_invalid_input)
      .pipe(Err)?
  }

  resolve_dst_file_path(src_path, dst_path)
    .ok_or_else(|| io_invalid_input("Destination path should not be a directory"))
}

/// Decide the final destination path for a single file copy.
///
/// - If `dst_path` is a directory => append the source file name to it.
/// - _ => use `dst_path` as-is (it is already the final file path).
///
/// Returns `None` only when the source file name cannot be obtained
/// (e.g., `src_path` ends with `..`).
///
/// # Example
///
/// ```
///   use std::{borrow::Cow, path::Path};
///   use cardbox::imp_std::copy::file::resolve_dst_file_path;
///
///   let src = Path::new("./crate/Cargo.toml");
///   let dst = Path::new("/tmp");
///   let final_path = Path::new("/tmp/Cargo.toml");
///
///   resolve_dst_file_path(src, dst).map(|v| assert_eq!(&v, final_path));
/// ```
pub fn resolve_dst_file_path<'a>(
  src_path: &'a Path,
  dst_path: &'a Path,
) -> Option<Cow<'a, Path>> {
  match dst_path.is_dir() {
    true => src_path
      .file_name()
      .map(|x| dst_path.join(x))
      .map(Cow::from),
    _ => Some(dst_path.into()),
  }
}

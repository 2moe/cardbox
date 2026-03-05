use std::path::Path;

use tap::Pipe;

use crate::imp_std::common::eputs;

/// Splits the last argument as the destination path and returns the remaining
/// source arguments.
///
/// # Panics
///
/// Panics if `args` is empty.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use cardbox::cli::copy::split_last_path;
///
/// let args = ["a.txt", "b.txt", "/tmp"];
/// let (dst, srcs) = split_last_path(&args);
///
/// assert_eq!(dst, Path::new("/tmp"));
/// assert_eq!(srcs, &["a.txt", "b.txt"]);
/// ```
pub fn split_last_path(args: &[String]) -> (&Path, &[String]) {
  args
    .split_last()
    .expect("Failed to get destination path")
    .pipe(|(d, s)| (Path::new(d), s))
}

/// Prints the path to stderr.
pub fn eputs_path(p: &Path) -> std::io::Result<()> {
  p.to_string_lossy()
    .as_bytes()
    .pipe(eputs)?;
  Ok(())
}

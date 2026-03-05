use std::path::Path;

use tap::Pipe;

use crate::imp_std::common::eputs;

/// Splits the last path from the arguments.
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

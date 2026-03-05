use std::path::Path;

use cardbox::imp_std::common::eputs;
use tap::Pipe;

#[cfg(feature = "copy-file")]
pub(crate) mod copy_file;

#[cfg(feature = "copy-all")]
pub(crate) mod copy_all;

fn split_last_path(args: &[String]) -> (&Path, &[String]) {
  args
    .split_last()
    .expect("Failed to get destination path")
    .pipe(|(d, s)| (Path::new(d), s))
}

fn eputs_path(p: &Path) -> std::io::Result<()> {
  p.to_string_lossy()
    .as_bytes()
    .pipe(eputs)?;
  Ok(())
}

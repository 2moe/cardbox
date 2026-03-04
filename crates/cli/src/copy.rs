use std::io;

use cardbox::imp_std::common::{eputs, puts};
use tap::Pipe;

fn copy_file_help() -> io::Result<()> {
  "
Usage:
  "
  .pipe(puts)?;

  Ok(())
}

pub(crate) fn copy_file(_args: Option<&[String]>) -> io::Result<()> {
  eputs("copy_file")?;
  Ok(())
}

#[cfg(feature = "copy-all")]
pub(crate) fn copy_all(_args: Option<&[String]>) -> io::Result<()> {
  eputs("copy_all")?;
  Ok(())
}

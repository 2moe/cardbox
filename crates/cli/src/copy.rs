use std::io;

use cardbox::imp_std::common::eputs;

#[cfg(feature = "copy-file")]
pub(crate) mod copy_file;

#[cfg(feature = "copy-all")]
pub(crate) fn copy_all(_args: Option<&[String]>) -> io::Result<()> {
  eputs("copy_all")?;
  Ok(())
}

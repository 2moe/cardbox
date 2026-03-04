use std::io;

use cardbox::imp_std::common::eputs;

fn copy_file_help() {
  //
}

pub(crate) fn copy_file(args: Option<&[String]>) -> io::Result<()> {
  eputs("copy_file")?;
  Ok(())
}

#[cfg(feature = "copy-all")]
pub(crate) fn copy_all(args: Option<&[String]>) -> io::Result<()> {
  eputs("copy_all")?;
  Ok(())
}

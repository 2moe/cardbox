use core::ffi::CStr;

use rustix::{
  fs::{chmod, lstat, FileType, Mode, RawMode},
  io,
};

use crate::println;

/// Changes file or directory permissions.
pub fn change_mode(p: &CStr, mode: RawMode) -> io::Result<()> {
  let old_mode = lstat(p)?.st_mode;
  let ftype = FileType::from_raw_mode(old_mode);

  chmod(p, Mode::from_raw_mode(mode))?;
  println!(
    "{ftype:?}: {p:?}
    old-mode: {old_mode:o}
    new-mode: {mode:o}"
  );
  Ok(())
}

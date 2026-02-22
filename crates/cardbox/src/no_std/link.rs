use core::ffi::CStr;

use rustix::{
  fs::{self, symlink, unlink},
  io,
};

use crate::{common::CARDBOX_VER, eprintln, fs::concat_dst_path, println};

/// Creates a symlink or hardlink.
///
/// If dst is a directory, the filename of src is automatically appended to dst.
///
/// For example:
/// - src: **Cargo.lock**
/// - dst: **/tmp**
///
/// Then dst would become **/tmp/Cargo.lock**
///
/// Note: If an error occurs during the creation process, the old dst file will
/// be automatically deleted by calling `unlink(dst)`.
pub fn new_link(src: &CStr, dst: &CStr, is_symlink: bool) -> io::Result<()> {
  let dst = concat_dst_path(src, dst);
  println!("src: {src:?}\ndst: {dst:?}");
  let link = if is_symlink { symlink } else { fs::link };

  if link(src, dst.as_ref()).is_err() {
    eprintln!("[WARN] unlink the old file: {dst:?}");
    let _ = unlink(dst.as_ref());
    link(src, dst.as_ref())?;
  };

  println!(
    "completed: {dst:?} {symbol} {src:?}",
    symbol = if is_symlink { "-->" } else { "==>" }
  );
  Ok(())
}

pub fn show_link_help_info(len: usize, name: &str) {
  if len < 2 {
    println!(
      "Version: {CARDBOX_VER}\n\
    Usage:
    {name} [src-path] [dst-path]",
    );
    unsafe { libc::exit(0) }
  }
}

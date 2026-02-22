#![allow(dead_code)]
const PATH_SEPARATOR: char = '/';

use alloc::{borrow::Cow, format};
use core::ffi::CStr;

use rustix::{
  fd::OwnedFd,
  fs::{self, FileType, OFlags, RawMode},
  io,
  path::Arg,
};

use crate::eprintln;

/// Checks if the file is a symlink, and if so, calls `read_link()`.
pub fn check_and_read_link(path: &CStr) -> Option<Cow<CStr>> {
  let mode = fs::lstat(path).ok()?.st_mode;
  if FileType::from_raw_mode(mode) != FileType::Symlink {
    return None;
  }
  Some(read_link(path))
}

/// # Example
///
/// ```no_run
/// let _ = rustix::fs::symlink("Cargo.lock", "tmp.toml");
///
/// let symlink = c"tmp.toml";
/// let dst = cardbox::fs::read_link(symlink);
///
/// assert_eq!(dst.as_ref(), c"Cargo.lock");
/// ```
pub fn read_link(src: &CStr) -> Cow<'_, CStr> {
  match fs::readlink(src, [0; 32]) {
    Ok(p) => Cow::from(p),
    _ => Cow::Borrowed(src),
  }
}

pub fn is_dir(mode: RawMode) -> bool {
  // (mode & S_IFMT) == S_IFDIR
  FileType::from_raw_mode(mode) == FileType::Directory
}

/// Similarly: `Path::new(path).is_dir()`
fn is_path_dir(path: &CStr) -> bool {
  match fs::stat(path) {
    Ok(stat) => {
      let mode = stat.st_mode;
      let is_d = is_dir(mode);
      eprintln!("[INFO] path: {path:?}, st_mode: 0o{mode:o}, is_dir: {is_d}");
      is_d
    }
    _ => false,
  }
}

/// Assume that there is:
///
/// ```sh
///     copy a.txt /tmp
/// ```
///
/// where **a.txt** is src and **/tmp** is dst.
///
/// `concat_dst_path("a.txt", "/tmp")` converts `/tmp` to `/tmp/a.txt`
pub(crate) fn concat_dst_path<'d>(src: &CStr, dst: &'d CStr) -> Cow<'d, CStr> {
  if !is_path_dir(dst) {
    return Cow::from(dst);
  }

  let src_lossy = src.to_string_lossy();
  const ERR: &str = "Invalid UTF-8 path";

  // src.file_name()
  let fname = last_path_name(&src_lossy);

  // src.join(dst)
  format!(
    "{}/{fname}",
    dst
      .as_str()
      .expect(ERR)
      .trim_end_matches('/')
  )
  .into_c_str()
  .expect(ERR)
}

fn last_path_name(p: &str) -> &str {
  p.split(
    #[cfg(unix)]
    PATH_SEPARATOR,
    #[cfg(not(unix))]
    ['/', '\\'],
  )
  .next_back()
  .expect("Invalid Src File Path")
}

/// Similarly: `File::open(src_path)`
///
/// # Example
///
/// ```
/// use cardbox::fs::open_file;
/// let path = c".";
/// let file_descriptor = open_file(path);
/// ```
pub fn open_file(path: &CStr) -> io::Result<OwnedFd> {
  fs::open(path, OFlags::RDONLY, fs::Mode::empty()).inspect_err(|e| {
    eprintln!("[ERROR]: {e}");
    eprintln!("Failed to open the file: {path:?}")
  })
}

#[cfg(test)]
mod tests {
  use libc_print::libc_dbg;
  // use crate::println;

  #[test]
  fn open() {
    use crate::fs::open_file;
    let path = c".";
    let file_descriptor = open_file(path);
    let _ = libc_dbg!(file_descriptor);
  }
}

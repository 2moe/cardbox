use core::ffi::CStr;
use std::os::fd::OwnedFd;

use rustix::{
  // fs::{fcntl_lock, FlockOperation},
  fs::{self, OFlags},
  io,
  stdio,
};

/// Similarly: `File::open(src_path)`
///
/// # Example
///
/// ```ignore
/// use cardbox::fs::open_file_read_only;
/// let file_descriptor = open_file_read_only(c"Cargo.toml");
/// ```
pub fn open_file_read_only(path: &CStr) -> io::Result<OwnedFd> {
  fs::open(path, OFlags::RDONLY, fs::Mode::empty())
  // .inspect_err(|e| {
  //   eprintln!("[ERROR]: {e}");
  //   eprintln!("Failed to open the file: {path:?}")
  // })
}

/// It first calls `io::read()` to read the data and then writes it to
/// **stdout**.
///
/// Note: If files is `&[c""]`, it will read from stdin.
///
///
///
/// # Example
///
/// ```no_run
/// use cardbox::cat::io_copy_to_stdout;
///
/// let files = [c"Cargo.toml"];
///
/// let _ = io_copy_to_stdout(&files);
/// ```
pub fn io_copy_to_stdout(files: &[&CStr]) -> io::Result<()> {
  let mut buf = [0; 16 * 1024];
  let stdout = unsafe { stdio::take_stdout() };
  // fcntl_lock(&mut stdout, FlockOperation::LockExclusive)?;

  for fd_res in files
    .iter()
    .map(|x| match x.is_empty() {
      true => Ok(unsafe { stdio::take_stdin() }),
      _ => open_file_read_only(x),
    })
  {
    let fd = fd_res?;
    while let read_size @ 1.. = io::read(&fd, &mut buf)? {
      io::write(&stdout, unsafe { buf.get_unchecked(..read_size) })?;
    }
  }

  Ok(())
}

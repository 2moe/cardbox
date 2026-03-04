use std::io::IoSlice;

use rustix::io;

/// filesystem I/O api
#[cfg(feature = "rustix_fs")]
pub mod fs;

pub mod consts;

pub fn puts(buf: &[u8]) -> io::Result<usize> {
  if buf.is_empty() {
    return Ok(0);
  }
  let out = unsafe { rustix::stdio::take_stdout() };
  io::writev(&out, &[buf, b"\n"].map(IoSlice::new))
}

#[cfg(test)]
mod tests {
  use rustix::{fd::AsFd, io};

  use super::*;

  #[ignore]
  #[test]
  fn test_print_to_stdout() {
    let stdout = unsafe { rustix::stdio::stdout() };
    let out = stdout;

    io::write(out, b"Hello, world!\n").unwrap();
    // let out = unsafe { stdout().as_fd() };

    // .write_all(b"Hello, world!\n")
    // .unwrap();
  }
}

/// filesystem I/O api
#[cfg(feature = "fs")]
pub mod fs;

pub mod common;

pub mod consts;

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

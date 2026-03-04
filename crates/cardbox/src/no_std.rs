/// filesystem I/O api
#[cfg(feature = "rustix_fs")]
pub mod fs;

mod common;
pub use common::puts;
#[cfg(feature = "list")]
pub use common::readable_unit;
// === UNIX only ===
#[cfg(unix)]
#[cfg(feature = "uts_name")]
pub use rustix::system::uname;
// =========

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]
  #[test]
  fn test_print_to_stdout() {
    {
      let _ = puts(b"Hello");
    }
    let _ = puts(b"World");
  }

  #[ignore]
  #[test]
  fn show_uname() {
    let uname = super::uname();
    let rel = uname.release();
    let _ = puts(rel.to_bytes());
  }
}

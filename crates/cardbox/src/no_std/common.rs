use std::io::IoSlice;

use rustix::io;

// pub const fn yes() -> bool {
//   true
// }
// pub const fn no() -> bool {
//   false
// }

pub fn puts(buf: &[u8]) -> io::Result<usize> {
  if buf.is_empty() {
    return Ok(0);
  }
  let out = unsafe { rustix::stdio::take_stdout() };
  io::writev(&out, &[buf, b"\n"].map(IoSlice::new))
}

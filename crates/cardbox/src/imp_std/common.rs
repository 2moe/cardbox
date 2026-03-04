use std::io::{self, IoSlice, Write};

pub fn concat_newline(bytes: &[u8]) -> [std::io::IoSlice<'_>; 2] {
  [bytes, b"\n"].map(IoSlice::new)
}

/// - pseudocode: "{s}\n" |> stdio.write
/// - similar to: `println!("{s}")`
pub fn puts<S: AsRef<[u8]>>(s: S) -> io::Result<usize> {
  let bufs = concat_newline(s.as_ref());

  io::stdout().write_vectored(&bufs)
}

/// - pseudocode: "{s}\n" |> stderr.write
/// - similar to: `eprintln!("{s}")`
pub fn eputs<S: AsRef<[u8]>>(s: S) -> io::Result<usize> {
  let bufs = concat_newline(s.as_ref());

  io::stderr().write_vectored(&bufs)
}

/// - similar to: `eprint!("{s}")`
pub fn eprint<S: AsRef<[u8]>>(s: S) -> io::Result<()> {
  io::stderr().write_all(s.as_ref())
}

#[cfg(test)]
mod tests {
  use super::*;

  // #[test]
  // #[ignore]
  // fn show_exe_stem() {
  //   let _ = dbg!(first_arg_stem());
  // }
}

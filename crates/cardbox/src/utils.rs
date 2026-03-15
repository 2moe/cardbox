use std::io::{self, IoSlice, Write};

/// Converts a byte count into a human‑readable IEC unit (base‑1024).
///
/// Uses the following binary units:
///
/// - B   (Bytes)
/// - KiB (Kibibytes) = 2^10 = 1,024 Bytes
/// - MiB (Mebibytes) = 2^20 = 1,048,576 Bytes
/// - GiB (Gibibytes) = 2^30 = 1,073,741,824 Bytes
/// - TiB (Tebibytes) = 2^40 = 1,099,511,627,776 Bytes
/// - PiB             = 2^50
///
/// The returned value is truncated using integer division.
///
/// # Example
///
/// ```
/// use cardbox::no_std::readable_unit;
///
/// assert_eq!(readable_unit(1023), (1023.0, "B"));
/// assert_eq!(readable_unit(1024), (1.0, "KiB"));
/// assert_eq!(readable_unit(1524), (1.48828125/*  */, "KiB"));
/// assert_eq!(readable_unit(5 * 1024 * 1024), (5.0, "MiB"));
/// assert_eq!(readable_unit(3 * 1024_i64.pow(3)), (3.0, "GiB"));
/// assert_eq!(readable_unit(0), (0.0, "B"));
/// ```
// It's interesting to note that using the f32/f64 types here results in a 20K
// larger binary file.
pub fn readable_unit(bytes: i64) -> (f64, &'static str) {
  ["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
    .iter()
    .enumerate()
    .map(|(i, &unit)| (bytes as f64 / 1024f64.powi(i as _), unit))
    .take_while(|(size, _)| size.abs() >= 1.0)
    .last()
    .unwrap_or((bytes as _, "B"))
}

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

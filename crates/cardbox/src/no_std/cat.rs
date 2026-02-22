use alloc::{format, string::String};
use core::ffi::CStr;

use rustix::{
  // fs::{fcntl_lock, FlockOperation},
  io,
  stdio,
};

use crate::{common::CARDBOX_VER, eprintln, fs::open_file, println};

/// It first calls `io::read()` to read the data and then writes it to
/// **stdout**.
///
/// Note: If files is `&[c""]`, it will read from stdin.
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
  let mut buf = [0; 64 * 1024];
  let stdout = unsafe { stdio::take_stdout() };
  // fcntl_lock(&mut stdout, FlockOperation::LockExclusive)?;

  for fd_res in files
    .iter()
    .map(|x| match x.is_empty() {
      true => Ok(unsafe { stdio::take_stdin() }),
      _ => open_file(x),
    })
  {
    let fd = fd_res?;
    while let read_size @ 1.. = io::read(&fd, &mut buf)? {
      io::write(&stdout, unsafe { buf.get_unchecked(..read_size) })?;
    }
  }

  Ok(())
}

/// It calls `io::read()` to read the data, then calls `core::str::from_utf8()`
/// to convert the data to UTF-8 str.
///
/// Panic occurs when the data doesn't contain valid UTF-8.
///
/// Finally the str is converted to bytes and written to stdout.
///
/// If show_line: true, the number of lines is also appended.
///
/// If `convert_to_u8str_to_stdout(&[file1, file2], true)` is called, the number
/// of lines output is the total number of lines in all text files.
///
/// See also: [io_copy_to_stdout()](io_copy_to_stdout)
///
/// # Example
///
/// ```
/// use cardbox::cat::convert_to_u8str_to_stdout;
///
/// let files = [c"rust-toolchain.toml", c"Cargo.toml"];
///
/// let show_line = true;
///
/// let _ = convert_to_u8str_to_stdout(&files, show_line);
/// ```
///
/// ## Output:
///
/// ```toml
/// 1  [toolchain]
/// 2  channel = "nightly"
/// 3  [package]
/// 4  name = "cardbox"
/// 5  version = "0.0.0"
/// 6  edition = "2021"
/// ...
/// ```
///
/// The 1st & 2nd lines are the contents of file1, the 3rd and subsequent lines
/// are the contents of file2.
pub fn convert_to_u8str_to_stdout(
  files: &[&CStr],
  show_line: bool,
) -> io::Result<()> {
  const KIB_64: usize = 64 * 1024;
  let mut buf = [0; KIB_64];
  let stdout = unsafe { stdio::take_stdout() };
  let mut full_text =
    if show_line { String::with_capacity(KIB_64) } else { Default::default() };
  let mut total_line_num = 0;

  for fd_res in files
    .iter()
    .map(|x| match x.is_empty() {
      true => Ok(unsafe { stdio::take_stdin() }),
      _ => open_file(x),
    })
  {
    let fd = fd_res?;

    while let read_size @ 1.. = io::read(&fd, &mut buf)? {
      let utf8_str = core::str::from_utf8(unsafe { buf.get_unchecked(..read_size) })
        .map_err(|e| {
          eprintln!("[ERROR] {e}");
          io::Errno::INVAL
        })?;

      if show_line {
        full_text.push_str(utf8_str);
        continue;
      }
      io::write(&stdout, utf8_str.as_bytes())?;
    }
    if show_line {
      for line in full_text.lines() {
        total_line_num += 1;
        // println!("{total_line_num}  {line}")
        io::write(&stdout, format!("{total_line_num}  {line}").as_bytes())?;
      }
      full_text.clear();
    }
  }

  Ok(())
}

pub fn show_help_info() {
  println!(
    r#"Version: {CARDBOX_VER}
Note:
    1. `cats` converts file data to UTF-8 string, while `cat` does not.
    2. Panic occurs when [file] doesn't contain valid UTF-8.
    3. Only `cats` & `cat-str` support `show_line()` (-n flag)
Usage:
    cat [file]
    cats [file]
    cats -n [file]
    cat-str -n [file1] [file2]
    cat [file1] [file2] > [file3]"#,
  );
  unsafe { libc::exit(0) }
}

use alloc::borrow::Cow;
use core::ffi::CStr;

use rustix::{
  fd::OwnedFd,
  fs::{self, Mode, OFlags, RawMode},
  io::{self, Errno},
};

use crate::{
  common::{readable_unit, CARDBOX_VER},
  eprintln,
  fs::{check_and_read_link, concat_dst_path, open_file},
  println,
};

fn copy(src: &CStr, dst: &CStr) -> io::Result<()> {
  let src_path = check_and_read_link(src).unwrap_or_else(|| Cow::from(src));
  let src_fd = open_file(&src_path)?;
  let (st_mode, file_size) = get_src_metadata(&src_fd);

  let dst_path = concat_dst_path(src, dst);

  println!("copy: {src_path:?} -> {dst_path:?}");
  check_src_and_dst_file(&src_path, &dst_path);
  let dst_fd = create_dst(&dst_path, st_mode);

  #[cfg(not(target_os = "linux"))]
  fallback_to_io_read(&src_fd, &dst_fd, None);

  #[cfg(target_os = "linux")]
  if let Err(e) = fs::copy_file_range(&src_fd, None, &dst_fd, None, file_size) {
    fallback_to_io_read(&src_fd, &dst_fd, Some(e));
  }

  assert_dst_fsize(&dst_fd, file_size as _);
  Ok(())
}
/// Copies src_files to dst.
///
/// Where src can only be files, dst can be a file or a directory.
pub fn copy_files(src_files: &[&CStr], dst: &CStr) -> io::Result<()> {
  for &src in src_files {
    copy(src, dst)?
  }
  Ok(())
}

/// If src and dst are the same file, then panic.
fn check_src_and_dst_file(src_path: &CStr, dst_path: &CStr) {
  if *src_path == *dst_path
    || src_path.to_bytes()
      == dst_path
        .to_string_lossy()
        .trim_start_matches("./")
        .as_bytes()
  {
    panic!("`src` and `dst` cannot be the same file.")
  }
}

/// Gets the `st_mode` and `st_size` of src.
pub fn get_src_metadata(src_fd: &OwnedFd) -> (RawMode, usize) {
  match fs::fstat(src_fd) {
    Ok(m) => (m.st_mode, m.st_size as _),
    Err(_) => (0o755, 0),
  }
}

/// Success is indicated when src and dst have the same file size.
fn assert_dst_fsize(dst_fd: &OwnedFd, file_size: i64) {
  match fs::fstat(dst_fd) {
    Ok(m) => {
      let size = m.st_size;
      assert_eq!(size, file_size);
      let (fsize, unit) = readable_unit(size);
      eprintln!("[INFO] File Size: {fsize} {unit}");
      println!("The file was successfully copied.")
    }
    _ => panic!("{COPY_ERR}"),
  }
}

const COPY_ERR: &str = "Failed to copy file";

/// On Linux, using `copy_file_range()` may result in cross filesystem/device
/// errors, so it needs to fallback to `io::read()` & `io::write()`.
///
/// Also, not all operating systems have `copy_file_range()`, so those systems
/// also use `io::read()` & `io::write()`.
fn fallback_to_io_read(src_fd: &OwnedFd, dst_fd: &OwnedFd, err: Option<Errno>) {
  if let Some(_e) = err {
    // eprintln!("[WARN] {COPY_ERR} via `copy_file_range()`: {e}");
    eprintln!("[WARN] Falling back to `io::read()`");
  }
  eprintln!("Start copying...");

  let mut buf = [0; 16 * 1024];
  while let Ok(read_size @ 1..) = io::read(src_fd, &mut buf) {
    io::write(dst_fd, unsafe { buf.get_unchecked(..read_size) })
      .expect("Failed to read src data");
  }
}

/// Similarly:
///
/// ```ignore
/// OpenOptions::new()
///     .write(true)
///     .create(true)
///     .truncate(true)
///     .mode(st_mode)
///     .open(dst_path)
/// ```
fn create_dst(dst_path: &CStr, st_mode: RawMode) -> OwnedFd {
  fs::open(
    dst_path,
    OFlags::WRONLY | OFlags::CREATE | OFlags::TRUNC,
    Mode::from_raw_mode(st_mode as _),
  )
  .unwrap_or_else(|e| {
    eprintln!("[ERROR]: {e}");
    panic!("Failed to create the file: {dst_path:?}")
  })
}

pub fn show_help_info(len: usize) {
  if len >= 2 {
    return;
  }

  println!(
    r#"Version: {CARDBOX_VER}
Usage:
    copy [src-file] [dst-file]
    copy [src-file] [dst-dir]
    copy [src1] [src2] [src...] [dst]"#,
  );
  unsafe { libc::exit(0) }
}

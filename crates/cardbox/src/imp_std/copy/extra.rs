use std::path::Path;

pub use fs_extra;
use fs_extra::{dir::CopyOptions, file::CopyOptions as FileCopyOptions};

/// Recursively copy an entire directory tree to a new location.
///
/// # Example
///
/// ```ignore
/// use cardbox::imp_std::copy::copy_all;
///
/// copy_all("/tmp/source", "/tmp/dest", true).expect("copy failed");
/// ```
pub fn copy_all<P: AsRef<Path>, Q: AsRef<Path>>(
  from: P,
  to: Q,
) -> fs_extra::error::Result<u64> {
  let options = cp_rf_dir_options();
  fs_extra::dir::copy(from, to, &options)
}

pub fn cp_rf_dir_options() -> CopyOptions {
  CopyOptions {
    overwrite: true,
    copy_inside: true,
    ..Default::default()
  }
}

pub fn cp_file_options() -> FileCopyOptions {
  FileCopyOptions {
    overwrite: true,
    ..Default::default()
  }
}

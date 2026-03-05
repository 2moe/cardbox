use std::path::Path;

use fs_extra::dir::CopyOptions;
pub mod copy_file;

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
  overwrite: bool,
) -> fs_extra::error::Result<u64> {
  let options = CopyOptions {
    overwrite,
    copy_inside: true,
    ..Default::default()
  };

  fs_extra::dir::copy(from, to, &options)
}

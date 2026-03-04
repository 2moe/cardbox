use std::path::Path;

pub use fs_extra;
use fs_extra::dir::CopyOptions;

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

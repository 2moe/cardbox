use std::{
  fs::{self, File},
  io::{self, BufReader, BufWriter},
  path::Path,
};

use tap::Pipe;

use crate::{copy::file::validate_and_resolve_dst_path, utils::eputs};

#[cfg(feature = "link")]
pub mod link;

/// Creates a new buffered writer for a path.
///
/// # Example
///
/// ```ignore
/// use std::io::Write;
/// use cardbox::imp_std::fs::create_a_new_buf_writer;
///
/// let mut w = create_a_new_buf_writer("/tmp/demo.txt")?;
/// w.write_all(b"hello world")?;
/// w.flush()?;
/// # std::fs::remove_file("/tmp/demo.txt"); // keep the example clean
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn create_a_new_buf_writer<P: AsRef<Path>>(
  path: P,
) -> io::Result<BufWriter<File>> {
  create_a_new_file(path)?
    .pipe(BufWriter::new)
    .pipe(Ok)
}

pub fn wrap_buf_reader<P: AsRef<Path>>(path: P) -> io::Result<BufReader<File>> {
  File::open(path)?
    .pipe(BufReader::new)
    .pipe(Ok)
}

/// OpenOptions: create + write + truncate + open
pub fn create_a_new_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
  File::options()
    .create(true)
    .write(true)
    .truncate(true)
    .open(path)
}

pub fn rename_path(src_path: &Path, dst_path: &Path) -> Result<(), io::Error> {
  let dst_path = validate_and_resolve_dst_path(src_path, dst_path)?;
  if dst_path.exists()
    && src_path.is_file()
    && let Err(e) = fs::remove_file(&dst_path)
  {
    eputs("[WARN] Failed to remove existing file")?;
    eputs(e.to_string())?;
  }
  fs::rename(src_path, dst_path)
}

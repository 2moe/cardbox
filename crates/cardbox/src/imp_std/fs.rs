use std::{
  fs::File,
  io::{self, BufWriter},
  path::Path,
};

use tap::Pipe;

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

/// OpenOptions: create + write + truncate + open
pub fn create_a_new_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
  File::options()
    .create(true)
    .write(true)
    .truncate(true)
    .open(path)
}

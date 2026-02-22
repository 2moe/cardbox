use std::path::Path;

use testutils::tap::Pipe;

mod cardbox_target;

fn manifest_dir() -> &'static Path {
  env!("CARGO_MANIFEST_DIR").pipe(Path::new)
}

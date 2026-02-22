//! ```ignore, sh
//! cargo open-doc
//! ```
use std::io;

use testutils::{
  os_cmd::{RunnableCommand, presets::CargoDoc},
  tap::Pipe,
};

#[ignore]
#[test]
fn build_and_open_rust_doc() -> io::Result<()> {
  [
    "cardbox",
    // "cardbox-target",
  ]
  .as_ref()
  .pipe(build_doc)
}

fn build_doc(crates: &[&str]) -> io::Result<()> {
  let build = |&pkg| {
    CargoDoc::default()
      .with_pkg(pkg)
      .with_enable_private_items(false)
      // .with_open(false)
      .run()
  };

  crates
    .iter()
    .try_for_each(build)
}

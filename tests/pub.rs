use std::{env::set_current_dir, io, path::Path};

use testutils::{
  os_cmd::{Runner, fmt_compact},
  tap::{Conv, Pipe},
};

#[ignore]
#[test]
fn cargo_publish() -> io::Result<()> {
  let workdir = env!("CARGO_MANIFEST_DIR").pipe(Path::new);
  [
    // "cardbox",
    "cardbox-target",
    // "cli",
  ]
  .iter()
  .map(|x| fmt_compact!("crates/{x}"))
  .try_for_each(|d| {
    workdir
      .join(d)
      .pipe(set_current_dir)?;

    "cargo publish --registry crates-io"
      .conv::<Runner>()
      .run_command()
  })
}

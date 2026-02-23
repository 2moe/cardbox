use std::{env::set_current_dir, io, path::Path};

use testutils::{
  os_cmd::{MiniStr, Runner, fmt_compact},
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
  .try_for_each(|dir| {
    workdir
      .join(&dir)
      .pipe(set_current_dir)?;

    "cargo publish --registry crates-io"
      .split_ascii_whitespace()
      .map(MiniStr::from)
      .chain(
        dir
          .ends_with("/cardbox-target")
          .then(|| "--no-verify".into()),
      )
      .collect::<Box<_>>()
      .conv::<Runner>()
      .run_command()
  })
}

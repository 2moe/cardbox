//! ```ignore, sh
//! cargo open-doc
//! ```
use std::io;

use testutils::{
  os_cmd::{self, RunnableCommand, Runner, presets::CargoDoc},
  print_ext::normal::edbg,
  tap::{Pipe, Tap},
};

#[ignore]
#[test]
fn build_and_serve_doc() -> io::Result<()> {
  let crates = [
    "=",
    // "=-target",
    // "=cli",
  ]
  .map(|s| s.replacen("=", "cardbox", 1))
  .tap(edbg);

  crates
    .iter()
    .map(AsRef::as_ref)
    .try_for_each(build_doc)?;

  crates
    .first()
    .map(|p| serve_doc(p))
    .transpose()?;

  Ok(())
}

fn build_doc(pkg: &str) -> io::Result<()> {
  CargoDoc::default()
    .with_pkg(pkg)
    .with_enable_private_items(false)
    // .with_all_features(false)
    .with_open(false)
    .into_tinyvec()
    .pipe(os_cmd::run)
}

/// Uses `miniserve` instead of `.with_open(true)` on CargoDoc to ensure
/// compatibility with Remote-SSH.
#[ignore]
// #[test]
fn serve_doc(pkg: &str) -> io::Result<()> {
  let dir = env!("CARGO_MANIFEST_DIR");

  format!(
    "miniserve
    {dir:?}/target/doc
    --index {pkg}/index.html
    ",
  )
  .pipe_deref(Runner::from)
  .tap(|_| eprintln!("\x1b[35mhttp://127.0.0.1:8080/{pkg}/index.html\x1b[0m"))
  .run()
  .inspect_err(|e| eprintln!("{e:?};\n cargo binstall miniserve"))
}

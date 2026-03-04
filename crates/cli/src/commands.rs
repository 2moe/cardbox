use std::io::{self, Write, stdout};

use cardbox::imp_std::common::{eputs, puts};
use tap::Pipe;

pub(crate) fn all_available_commands() -> Box<[&'static str]> {
  vec![
    #[cfg(feature = "copy-file")]
    "copy-file",
    //
    #[cfg(feature = "copy-all")]
    "copy-all",
    //
    #[cfg(feature = "target")]
    "target",
    //
    #[cfg(feature = "uts-name")]
    "uts-name",
    //
    #[cfg(feature = "list")]
    "list",
    //
    #[cfg(feature = "rename")]
    "rename",
    //
    #[cfg(feature = "run-command")]
    "run-command",
  ]
  .into()
}

pub(crate) fn help_info() -> io::Result<()> {
  "CardBox: CLI utilities purpose-built for scratch container environments

Usage: cardbox [sub-command] [arguments]...
  or: cardbox [option]

e.g., cardbox --list

Options:

  --list
    List all commands, one per line."
    .pipe(eputs)?;

  #[cfg(feature = "serde")]
  eputs(
    "  --list-json
    List all commands in JSON format.",
  )?;

  "
Available sub-commands:"
    .pipe(eputs)?;

  all_available_commands()
    .join(", ")
    .pipe_deref(puts)?;

  Ok(())
}

pub(crate) fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub(crate) fn display_version() -> io::Result<()> {
  puts(version())?;
  Ok(())
}

pub(crate) fn list_all_commands() -> io::Result<()> {
  let mut lock = stdout().lock();

  for data in all_available_commands()
    .iter()
    .flat_map(|s| [s.as_bytes(), b"\n"])
  {
    lock.write_all(data)?;
  }

  lock.flush()
}

#[cfg(feature = "serde")]
pub(crate) fn list_all_commands_json() -> io::Result<()> {
  serde_json::json!({
    "commands": all_available_commands()
  })
  .to_string()
  .pipe_deref(puts)?;

  Ok(())
}

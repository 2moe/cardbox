use std::io::{self, Write, stdout};

use cardbox::imp_std::common::{eprint, eputs, puts};
use tap::Pipe;

pub(crate) fn all_available_commands() -> Box<[&'static str]> {
  // #[cfg(unix)]
  // #[cfg(feature = "uts-name")]
  // "uts-name",
  vec![
    #[cfg(feature = "copy-file")]
    "copy-file",
    //
    #[cfg(feature = "copy-all")]
    "copy-all",
    //
    // target + utsname
    #[cfg(feature = "target")]
    "target",
    //
    #[cfg(feature = "list")]
    "list",
    //
    #[cfg(feature = "rename")]
    "rename",
    //
    #[cfg(feature = "run-cmd")]
    "run-cmd",
  ]
  .into()
}

pub(crate) fn help_info() -> io::Result<()> {
  "CardBox: CLI utilities purpose-built for scratch container environments

Usage: cardbox [sub-command] [arguments]...
  or: cardbox [option]

e.g.,
  cardbox target
  cardbox --version

Options:
  --V
    Display the version: "
    .pipe(eprint)?;

  version().pipe(eputs)?;

  #[cfg(feature = "serde")]
  "  --version
    Display the version in JSON format."
    .pipe(eputs)?;

  "  -L
    List all commands, one per line."
    .pipe(eputs)?;

  #[cfg(feature = "serde")]
  "  --list
    List all commands in JSON format."
    .pipe(eputs)?;

  eputs("\nAvailable SubCommands:")?;

  all_available_commands()
    .join(", ")
    .pipe(puts)?;

  Ok(())
}

pub(crate) fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub(crate) fn display_version() -> io::Result<()> {
  puts(version())?;
  Ok(())
}

#[cfg(feature = "serde")]
pub(crate) fn display_version_in_json() -> io::Result<()> {
  let feats =
    env!("__cardbox_cli_features").pipe(serde_json::from_str::<Vec<&str>>)?;
  let wrap_some = |v| match v {
    "" => None,
    v => v.into(),
  };

  let commit_hash = env!("__cardbox_cli_commit_hash").pipe(wrap_some);
  let build_time = env!("__cardbox_cli_build_time").pipe(wrap_some);
  let host = env!("__cardbox_cli_host").pipe(wrap_some);

  serde_json::json!({
    "version": version(),
    "cargo_features": feats,
    "commit_hash": commit_hash,
    "build_time": build_time,
    "build_host": host,
    "target": cardbox::consts::target(),
  })
  .pipe_ref(serde_json::to_string_pretty)?
  .pipe(puts)?;

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
pub(crate) fn list_all_commands_in_json() -> io::Result<()> {
  serde_json::json!({
    "commands": all_available_commands()
  })
  .to_string()
  .pipe(puts)?;

  Ok(())
}

pub(crate) fn contains_help(args: &[String]) -> bool {
  args
    .iter()
    .any(|x| ["-h", "help", "--help"].contains(&x.as_ref()))
}

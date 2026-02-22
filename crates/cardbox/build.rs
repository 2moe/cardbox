use std::{
  env,
  io::{self, Write},
};

use compact_str::{CompactString as MiniStr, format_compact as fmt_compact};

fn main() {
  set_target_envs().expect("Failed to set envs")
}

fn set_target_envs() -> io::Result<()> {
  let target_vars = ["FAMILY", "OS", "ARCH", "POINTER_WIDTH", "ENDIAN", "FEATURE"]
    .iter()
    .map(|name| (name, fmt_compact!("CARGO_CFG_TARGET_{name}")));

  let direct_vars = ["TARGET", "PROFILE", "CARGO_ENCODED_RUSTFLAGS"]
    .iter()
    .map(|name| (name, MiniStr::const_new(name)));

  let env_err = |e| {
    io::Error::other(format!(
      "Failed to get env: CARGO_CFG_TARGET_*;
      VarError: {e}"
    ))
  };

  {
    let mut lock = io::stdout().lock();
    target_vars
      .chain(direct_vars)
      .try_for_each(|(name, env_name)| {
        let val = env::var(env_name.as_str()).map_err(env_err)?;
        writeln!(&mut lock, "cargo::rustc-env=__CARDBOX_CFG_{name}={val}")
      })?;

    lock.flush()
  }
}

use std::{
  env,
  io::{self, Write},
};

use testutils::cargo_cfg;

fn main() {
  set_target_envs().expect("Failed to set envs")
}

// https://doc.rust-lang.org/cargo/reference/environment-variables.html
fn set_target_envs() -> io::Result<()> {
  let env_vars = [
    ("family", cargo_cfg!(target_family)),
    ("os", cargo_cfg!(target_os)),
    ("arch", cargo_cfg!(target_arch)),
    ("vendor", cargo_cfg!(target_vendor)),
    ("env", cargo_cfg!(target_env)),
    ("abi", cargo_cfg!(target_abi)),
    ("pointer_width", cargo_cfg!(target_pointer_width)),
    ("endian", cargo_cfg!(target_endian)),
    ("feature", cargo_cfg!(target_feature)),
    ("cargo_feature", cargo_cfg!(feature)),
    ("target", env::var("TARGET")),
    // ("profile", env::var("PROFILE")),
    ("encoded_rust_flags", env::var("CARGO_ENCODED_RUSTFLAGS")),
  ];

  let env_err = |e| {
    io::Error::other(format!(
      "Failed to get env: CARGO_CFG_TARGET_*;
      VarError: {e}"
    ))
  };

  {
    let mut lock = io::stdout().lock();
    for (k, value) in env_vars {
      let v = value.map_err(env_err)?;
      writeln!(&mut lock, "cargo::rustc-env=__cardbox_cfg_{k}={v}")?;
    }
    lock.flush()
  }
}

// https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::{env, io};

use testutils::cargo_cfg;

fn main() {
  update_rustc_link_cfg();
  set_target_env().expect("Failed to update env")
}

fn update_rustc_link_cfg() {
  // linux
  if let Ok("linux") = cargo_cfg!(target_os).as_deref() {
    "cargo:rustc-link-arg=-nostartfiles".puts();
    "cargo:rustc-link-arg=-Wl,-e,_start".puts();
    if let Ok("musl") = cargo_cfg!(target_env).as_deref() {
      "cargo::rustc-link-lib=c".puts();
    }
  }
  // if let Ok("loongarch64-unknown-linux-musl" | "mipsel-unknown-linux-musl") =
  //   env::var("TARGET").as_deref()
  // { "cargo::rustc-link-arg=-Wl,-z,max-page-size=0x1000".puts() }

  // macOS
  if let Ok("macos") = cargo_cfg!(target_os).as_deref() {
    "cargo::rustc-link-lib=System".puts()
  }
}

fn set_target_env() -> io::Result<()> {
  let env_err =
    |e| io::Error::other(format!("Failed to get env var; VarError: {e}"));
  let target = env::var("TARGET").map_err(env_err)?;
  println!("cargo::rustc-env=__cardbox_cfg_target={target}");
  Ok(())
}
// ===========
pub trait Puts: core::fmt::Display {
  fn puts(&self) {
    println!("{self}")
  }
}
impl Puts for str {}

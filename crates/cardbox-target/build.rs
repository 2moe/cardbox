// https://doc.rust-lang.org/cargo/reference/build-scripts.html

use std::{env, io};

use testutils::cargo_cfg;

fn main() {
  update_rustc_link_cfg();
  set_target_env().expect("Failed to update env")
}

fn print_rs_link_arg(arg: &str) {
  println!("cargo:rustc-link-arg={arg}")
}

fn update_rustc_link_cfg() {
  let Ok(os) = cargo_cfg!(target_os) else {
    return;
  };

  match os.as_ref() {
    "linux" => {
      let _: [(); _] = ["-nostartfiles", "-Wl,-e,_start"].map(print_rs_link_arg);

      if let Ok("musl") = cargo_cfg!(target_env).as_deref() {
        print_rs_link_arg("c")
      }
    }

    "macos" => print_rs_link_arg("System"),
    _ => {}
  }

  // if let Ok("loongarch64-unknown-linux-musl") =
  //   env::var("TARGET").as_deref()
  // { print_rs_link_arg("-Wl,-z,max-page-size=0x1000") }
}

fn set_target_env() -> io::Result<()> {
  let env_err =
    |e| io::Error::other(format!("Failed to get env var; VarError: {e}"));

  let target = env::var("TARGET").map_err(env_err)?;
  println!("cargo::rustc-env=__cardbox_cfg_target={target}");
  Ok(())
}

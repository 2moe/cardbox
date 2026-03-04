use std::{borrow::Cow, env, io};

use testutils::{os_cmd::CommandSpawner, tap::Pipe};
use time::format_description::well_known::Rfc3339;

fn main() -> io::Result<()> {
  std::env::vars()
    .filter_map(|(k, _)| {
      k.strip_prefix("CARGO_FEATURE_")
        .map(|f| f.to_ascii_lowercase())
    })
    .collect::<Vec<_>>()
    .pipe_ref(serde_json::to_string)?
    .pipe(|value| update_cli_env("features", &value));

  set_git_commit_env();
  get_and_update_cli_env("HOST");
  set_utc_build_time_env();

  Ok(())
}

fn set_utc_build_time_env() {
  let now = time::OffsetDateTime::now_utc();
  let key = "build_time";

  match now.format(&Rfc3339) {
    Ok(t) => update_cli_env(key, &t),
    Err(_) => update_cli_env(key, ""),
  }
}

fn empty_cow_str() -> Cow<'static, str> {
  Cow::from("")
}

fn get_and_update_cli_env(key: &str) {
  let lower_key = key.to_ascii_lowercase();

  match env::var(key) {
    Ok(v) => v.into(),
    Err(e) => {
      cargo_warning(&e.to_string());
      empty_cow_str()
    }
  }
  .pipe(|v| update_cli_env(&lower_key, &v))
}

fn set_git_commit_env() {
  println!("cargo:rerun-if-changed=.git/HEAD");

  match "git rev-parse HEAD"
    .pipe(CommandSpawner::from)
    .capture_stdout()
  {
    Ok(s) => s.take_data().into(),
    _ => {
      cargo_warning(
        "Failed to get git commit hash.
      Please ensure git is installed and the repository is in a valid state.",
      );
      empty_cow_str()
    }
  }
  .pipe(|v| update_cli_env("commit_hash", &v))
}

fn cargo_warning(s: &str) {
  println!("cargo::warning={s}")
}
fn update_cli_env(key_suffix: &str, value: &str) {
  println!("cargo:rustc-env=__cardbox_cli_{key_suffix}={value}")
}

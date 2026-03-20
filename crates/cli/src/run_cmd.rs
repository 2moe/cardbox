use std::{
  io::{self},
  path::Path,
};

use cardbox::utils::{eputs, puts};
use tap::Pipe;

use crate::commands::is_first_help_flag;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  let path_strs = match args {
    Some(&[]) | None => return help(),
    Some(x) => x,
  };
  if is_first_help_flag(path_strs) {
    return help();
  }

  for path in path_strs.iter().map(Path::new) {
    if !path.exists() {
      eputs("[WARN] json/toml path does not exist, skipping: {path:?}")?;
      continue;
    }

    //
  }

  Ok(())
}

fn help() -> io::Result<()> {
  r#"
Usage:
      run-cmd [/path/to/toml-v1.1-file]
  OR: run-cmd [/path/to/json5-file]
  OR: run-cmd -

# ========================
sample.toml:
  # Only the cmd array is mandatory; all other fields (keys) are optional.
  # 只有 cmd 数组是必要的，其他字段都是可选的

  cmd = ["wc", "-m"]

  stdin_str = '''
  hello world
  '''

  # e.g., "-", "/path/to/stdin-data"
  stdin_path = ''

  # e.g., "/dev/null"
  stdout = "stdout.txt"
  stderr = "stderr.txt"

  timeout = 1000
  working_dir = "/tmp"

  [env]
  DATA_HOME = "/home/user/tmp"

// ========================
sample.json5:
  {
    cmd: [ "wc", "-m" ],
    stdin_str: "hello world",
    // stdin_path: "",
    stdout: "stdout.txt",
    stderr: "stderr.txt",
    timeout: 1000,
    working_dir: "/tmp",
    env: {
      DATA_HOME: "/home/user/tmp",
    }
  }
  "#
  .pipe(puts)?;
  Ok(())
}

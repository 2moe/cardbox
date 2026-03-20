use std::{
  fs,
  io::{self, Read},
  path::Path,
};

use cardbox::{
  run_cmd::{CmdData, ConfigFmt},
  utils::puts,
};
use tap::{Pipe, Tap};

use crate::commands::is_first_help_flag;

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  let path_strs = match args {
    Some(&[]) | None => return help(),
    Some(x) => x,
  };
  if is_first_help_flag(path_strs) {
    return help();
  }

  let mut stdin_found = false;
  for path_s in path_strs {
    let data = match stdin_found {
      false if path_s == "-" => {
        stdin_found = true;
        let mut buf = String::with_capacity(128);
        io::stdin().read_to_string(&mut buf)?;
        buf
      }
      _ if Path::new(path_s).exists() => fs::read_to_string(path_s)?,
      _ => {
        eprintln!("[WARN] json/toml path does not exist, skipping: {path_s}");
        continue;
      }
    };

    let fmt = {
      use ConfigFmt::*;
      match path_s.rsplit('.').next() {
        Some(x) if x.eq_ignore_ascii_case("toml") => Toml,
        Some(x)
          if x.eq_ignore_ascii_case("json") || x.eq_ignore_ascii_case("json5") =>
        {
          Json5
        }
        _ => Unknown,
      }
    };
    CmdData::new(&data, fmt)?
      // .tap(|s| eprintln!("{s:#?}"))
      .run()?
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

  # 2500 ms => 2.5s
  timeout = 2500
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
    timeout: 2500,
    working_dir: "/tmp",
    env: {
      DATA_HOME: "/home/user/tmp",
    }
  }
  "#
  .pipe(puts)?;
  Ok(())
}

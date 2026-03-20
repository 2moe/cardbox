use std::{
  borrow::Cow,
  io,
  path::PathBuf,
  process::{Command, Stdio},
};

pub use compact_str::CompactString as MiniStr;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tap::Pipe;
use tinyvec::TinyVec;

use crate::{
  copy::error::io_invalid_input,
  fs::{create_a_new_buf_writer, wrap_buf_reader},
};

pub type CommandArr = TinyVec<[MiniStr; 3]>;
pub type EnvMap = ahash::HashMap<MiniStr, MiniStr>;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CmdData {
  pub cmd: CommandArr,

  pub stdin_path: Option<PathBuf>,
  pub stdin_str: Option<MiniStr>,
  pub stdout: Option<PathBuf>,
  pub stderr: Option<PathBuf>,
  // pub stdout_and_stderr: Option<PathBuf>,
  // pub timeout: Option<i64>,
  pub working_dir: Option<PathBuf>,
  pub env: Option<EnvMap>,
}

enum StdioMode {
  Null,
  Inherit,
  Piped,
}

impl CmdData {
  pub fn run(&self) -> io::Result<()> {
    let stdin_carrier = self.stdin_carrierr();
    let Self {
      cmd,
      stdout,
      stderr,
      // timeout,
      working_dir,
      env,
      stdin_path,
      stdin_str,
    } = self;

    let Some((first_command, args)) = cmd.split_first() else {
      io_invalid_input("No command provided").pipe(Err)?
    };

    let mut command = first_command
      .as_str()
      .pipe(Command::new);
    command.args(args.iter().map(|s| s.as_str()));

    if let Some(p) = working_dir {
      command.current_dir(p);
    }

    let stdin_mode = match stdin_carrier {
      Some(_) => {
        command.stdin(Stdio::piped());
        StdioMode::Piped
      }
      _ => {
        command.stdin(Stdio::inherit());
        StdioMode::Inherit
      }
    };

    let stdout_mode = {
      let (m, s) = match path_to_str(stdout).as_deref() {
        Some("/dev/null") => (Stdio::null(), StdioMode::Null),
        Some(_) => (Stdio::piped(), StdioMode::Piped),
        _ => (Stdio::inherit(), StdioMode::Inherit),
      };
      command.stdout(m);
      s
    };

    let stderr_mode = {
      let (m, s) = match path_to_str(stderr).as_deref() {
        Some("/dev/null") => (Stdio::null(), StdioMode::Null),
        Some(_) => (Stdio::piped(), StdioMode::Piped),
        _ => (Stdio::inherit(), StdioMode::Inherit),
      };
      command.stderr(m);
      s
    };

    if let Some(e) = env {
      e.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .pipe(|m| command.envs(m));
    }

    let process = command.spawn()?;

    if let Some(mut stdin) = process.stdin
      && let StdioMode::Piped = stdin_mode
      && let Some(carrier) = stdin_carrier
    {
      use StdinCarrier::*;
      match carrier {
        File => {
          let mut reader = stdin_path
            .as_ref()
            .expect("stdin_path is None but carrier is File! ")
            .pipe(wrap_buf_reader)?;
          io::copy(&mut reader, &mut stdin)
        }
        Str => {
          let mut s = stdin_str
            .as_ref()
            .expect("stdin_str is None but carrier is Str! ")
            .as_bytes();
          io::copy(&mut s, &mut stdin)
        }
      }?;
    }

    if let Some(mut x) = process.stdout
      && let StdioMode::Piped = stdout_mode
      && let Some(out) = stdout
    {
      let mut writer = create_a_new_buf_writer(out)?;
      io::copy(&mut x, &mut writer)?;
    }

    if let Some(mut x) = process.stderr
      && let StdioMode::Piped = stderr_mode
      && let Some(out) = stderr
    {
      let mut writer = create_a_new_buf_writer(out)?;
      io::copy(&mut x, &mut writer)?;
    }

    Ok(())
  }

  pub fn stdin_carrierr(&self) -> Option<StdinCarrier> {
    let Self {
      stdin_path,
      stdin_str,
      ..
    } = self;

    if let Some(p) = stdin_path
      && p.exists()
      && p.is_file()
      && p
        .metadata()
        .is_ok_and(|x| x.len() != 0)
    {
      return Some(StdinCarrier::File);
    };

    match stdin_str.as_deref() {
      Some("") | None => None,
      Some(_) => StdinCarrier::Str.into(),
    }
  }
}

pub enum StdinCarrier {
  Str,
  File,
}

fn path_to_str(p: &Option<PathBuf>) -> Option<Cow<'_, str>> {
  p.as_ref()
    .map(|s| s.to_string_lossy())
}

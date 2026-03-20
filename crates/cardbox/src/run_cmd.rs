use std::{
  borrow::Cow,
  io,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  thread,
  time::{Duration, Instant},
};

pub use compact_str::CompactString as MiniStr;
use log::{info, warn};
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
  // todo: +$schema
  pub cmd: CommandArr,

  pub stdin_path: Option<PathBuf>,
  pub stdin_str: Option<MiniStr>,
  pub stdout: Option<PathBuf>,
  pub stderr: Option<PathBuf>,
  // pub stdout_and_stderr: Option<PathBuf>,

  //
  /// unit: ms; Some(3500) => timeout 3.5s
  pub timeout: Option<u64>,
  pub working_dir: Option<PathBuf>,
  pub env: Option<EnvMap>,
}

enum StdioMode {
  Null,
  Inherit,
  Piped,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ConfigFmt {
  Json5,
  Toml,
  #[default]
  Unknown,
}

impl CmdData {
  /// from_config_data
  pub fn new(data: &str, fmt: ConfigFmt) -> io::Result<Self> {
    use ConfigFmt::*;
    let input = data.trim();

    let deser_json5 = || json5::from_str(input).map_err(io::Error::other);
    let deser_toml = || toml::from_str(input).map_err(io::Error::other);

    match fmt {
      Json5 => deser_json5(),
      Toml => deser_toml(),
      Unknown if input.starts_with("{") => deser_json5(),
      // Try to parse as TOML first, then JSON5
      _ => match deser_toml() {
        Err(e) => {
          warn!("{e}\n  Failed to parse as TOML, trying JSON5...");
          deser_json5()
        }
        x => x,
      },
    }
  }

  pub fn run(self) -> io::Result<()> {
    let stdin_carrier = self.stdin_carrierr();
    let Self {
      cmd,
      stdout: stdout_file,
      stderr: stderr_file,
      timeout,
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
      let (m, s) = match path_to_str(&stdout_file).as_deref() {
        Some("/dev/null") => (Stdio::null(), StdioMode::Null),
        Some(_) => (Stdio::piped(), StdioMode::Piped),
        _ => (Stdio::inherit(), StdioMode::Inherit),
      };
      command.stdout(m);
      s
    };

    let stderr_mode = {
      let (m, s) = match path_to_str(&stderr_file).as_deref() {
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

    let start = Instant::now();
    let mut process = command.spawn()?;

    let child_stdin = process.stdin.take();
    let child_stdout = process.stdout.take();
    let child_stderr = process.stderr.take();

    // eprintln!("write stdin");
    let stdin_handle = match stdin_mode {
      StdioMode::Piped => thread::spawn(move || -> io::Result<()> {
        if let Some(mut stdin) = child_stdin
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
        Ok(())
      })
      .into(),
      _ => None,
    };

    // eprintln!("write stdout");
    let out_handle = match stdout_mode {
      StdioMode::Piped => thread::spawn(move || -> io::Result<()> {
        if let Some(mut child_out) = child_stdout
          && let Some(file) = stdout_file
        {
          let mut writer = create_a_new_buf_writer(file)?;
          io::copy(&mut child_out, &mut writer)?;
          writer.flush()?
        }
        Ok(())
      })
      .into(),
      _ => None,
    };

    // eprintln!("write stderr");
    let err_handle = match stderr_mode {
      StdioMode::Piped => thread::spawn(move || -> io::Result<()> {
        if let Some(mut child_out) = child_stderr
          && let Some(file) = stderr_file
        {
          let mut writer = create_a_new_buf_writer(file)?;
          io::copy(&mut child_out, &mut writer)?;
          writer.flush()?
        }
        Ok(())
      })
      .into(),
      _ => None,
    };

    let join_stdio_handle = || -> io::Result<()> {
      if let Some(h) = stdin_handle {
        h.join()
          .expect("Failed to join stdin thread")?;
      }
      if let Some(h) = out_handle {
        h.join()
          .expect("Failed to join stdout thread")?;
      }
      if let Some(h) = err_handle {
        h.join()
          .expect("Failed to join stderr thread")?;
      }
      Ok(())
    };

    if let Some(ms) = timeout
      && ms != 0
    {
      let timeout = Duration::from_millis(ms);
      loop {
        if let Some(s) = process.try_wait()? {
          join_stdio_handle()?;
          return match s.success() {
            true => Ok(()),
            _ => Err(io::Error::last_os_error()),
          };
        }
        if start.elapsed() >= timeout {
          warn!("timeout, killing child process");
          process.kill()?;
          let status = process.wait()?; // Reap zombie processes
          info!("Killed, status: {status}");
          // break;
          join_stdio_handle()?;
          return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
      }
    }
    process.wait()?;
    join_stdio_handle()?;
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

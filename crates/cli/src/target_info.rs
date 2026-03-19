use std::io;

use cardbox::utils::{eprint, eputs, puts};
use serde::Serialize;
use tap::Pipe;

use crate::commands::is_first_help_flag;

mod uts_name {
  use compact_str::CompactString as MiniStr;
  use serde::Serialize;

  #[cfg(not(unix))]
  #[derive(Serialize, Debug, Clone)]
  pub(crate) struct UtsName {}

  #[cfg(unix)]
  #[derive(Serialize, Debug, Clone)]
  pub(crate) struct UtsName {
    machine: MiniStr,
    sys_name: MiniStr,
    release: MiniStr,
    version: MiniStr,
    node_name: MiniStr,
  }

  #[cfg(unix)]
  impl UtsName {
    pub(crate) fn new() -> Self {
      use core::ffi::CStr;

      use compact_str::ToCompactString;
      use tap::Pipe;

      let to_str = |s: &CStr| {
        s.to_string_lossy()
          .to_compact_string()
      };

      let uname = cardbox::uname();
      let machine = uname.machine().pipe(to_str);
      let sys_name = uname.sysname().pipe(to_str);
      let release = uname.release().pipe(to_str);
      let version = uname.version().pipe(to_str);
      let node_name = uname.nodename().pipe(to_str);

      Self {
        machine,
        sys_name,
        release,
        version,
        node_name,
      }
    }

    pub(crate) fn take_inner(self, key: &str) -> Option<MiniStr> {
      let Self {
        machine,
        sys_name,
        release,
        version,
        node_name,
      } = self;

      match key {
        "m" | "machine" => machine.into(),
        "s" | "sys_name" | "sys-name" | "sysname" => sys_name.into(),
        "r" | "release" => release.into(),
        "v" | "version" => version.into(),
        "n" | "node_name" | "node-name" | "nodename" => node_name.into(),
        _ => None,
      }
    }
  }
}

type Sstr = &'static str;
type Features = Box<[&'static str]>;
#[derive(Serialize, Debug, Clone)]
struct TargetInfo {
  #[serde(skip_serializing_if = "Option::is_none", rename = "///")]
  comment: Option<Sstr>,

  target: Sstr,
  family: Sstr,
  os: Sstr,
  arch: Sstr,

  #[serde(skip_serializing_if = "str::is_empty")]
  vendor: Sstr,

  #[serde(skip_serializing_if = "str::is_empty")]
  env: Sstr,

  #[serde(skip_serializing_if = "str::is_empty")]
  abi: Sstr,

  pointer_width: Sstr,
  endian: Sstr,
  features: Features,
  cardbox_features: Features,

  #[serde(skip_serializing_if = "Option::is_none")]
  rust_flags: Option<Features>,

  #[serde(skip_serializing_if = "Option::is_none")]
  uts: Option<uts_name::UtsName>,
}

impl TargetInfo {
  fn with_comment(mut self, comment: Option<Sstr>) -> Self {
    self.comment = comment;
    self
  }
}

impl Default for TargetInfo {
  fn default() -> Self {
    use cardbox::consts::*;
    let split_comma = |s: &'static str| s.split(',').collect();

    Self {
      comment: target_info_comment().into(),
      family: target_family(),
      os: target_os(),
      arch: target_arch(),
      vendor: target_vendor(),
      env: target_env(),
      abi: target_abi(),
      pointer_width: target_pointer_width(),
      endian: target_endian(),
      features: target_feature().pipe(split_comma),
      cardbox_features: cargo_feature().pipe(split_comma),
      rust_flags: {
        let c = unsafe { char::from_u32_unchecked(0x1f) };
        let v = encoded_rust_flags()
          .split(c)
          .collect::<Vec<_>>();

        match v.as_slice() {
          &[] | &[""] => None,
          _ => Some(v.into()),
        }
      },
      target: target(),

      #[allow(unreachable_patterns)]
      uts: match () {
        #[cfg(unix)]
        _ => uts_name::UtsName::new().into(),
        _ => None,
      },
    }
  }
}

fn display_target_info_json(info: &TargetInfo) -> io::Result<()> {
  serde_json::to_string_pretty(info)
    .expect("Failed to serialize target info to JSON")
    .pipe(puts)?;

  Ok(())
}

pub(crate) fn run(args: Option<&[String]>) -> io::Result<()> {
  let info = TargetInfo::default();
  let help_in_json = || display_target_info_json(&info);

  // args is_empty() or None => help()
  let args = match args {
    Some(&[]) | None => return help_in_json(),
    Some(x) => x,
  };

  if is_first_help_flag(args) {
    return help();
  }

  let arg_2nd = args.get(1).map(|x| x.as_str());
  parse_first_two_args(&args[0], arg_2nd)?;

  Ok(())
}
fn help() -> io::Result<()> {
  puts(target_info_comment())?;
  show_available_keys()?;
  Ok(())
}

fn parse_first_two_args(arg_1st: &str, arg_2nd: Option<&str>) -> io::Result<()> {
  let info = TargetInfo::default().with_comment(None);

  match arg_1st {
    "/" => return display_target_info_json(&info),
    "///" => return help(),
    "f" | "family" => info.family,
    "o" | "os" => info.os,
    "a" | "arch" => info.arch,
    "v" | "vendor" => info.vendor,
    "env" => info.env,
    "abi" => info.abi,
    "p" | "pointer-width" | "pointer_width" => info.pointer_width,
    "e" | "endian" => info.endian,
    "fe" | "features" => {
      let v = info.features;
      println!("{v:?}");
      return Ok(());
    }
    "c" | "cardbox-features" | "cardbox_features" => {
      let v = info.cardbox_features;
      println!("{v:?}");
      return Ok(());
    }
    "r" | "rust_flags" | "rust-flags" | "rustflags" => {
      match info.rust_flags {
        Some(v) => println!("{v:?}"),
        _ => {
          eputs("[WARN] rust_flags is empty.")?;
        }
      }
      return Ok(());
    }
    "t" | "target" => info.target,
    #[cfg(unix)]
    "u" | "uts" => {
      let uts_data = info
        .uts
        .expect("Empty UtsName Data");

      match arg_2nd {
        Some(arg) => uts_data
          .take_inner(arg)
          .expect("Invalid key. Available: m | s | r | v | n")
          .pipe(puts),
        _ => serde_json::to_string_pretty(&uts_data)?.pipe(puts),
      }?;
      return Ok(());
    }
    _ => {
      eputs(r#"[WARN] Invalid key."#)?;
      show_available_keys()?;
      return Ok(());
    }
  }
  .pipe(puts)?;

  Ok(())
}
fn show_available_keys() -> io::Result<()> {
  r#"
  Available:
    /
    /// | help
    f   | family
    o   | os
    a   | arch
    v   | vendor
    env
    abi
    p   | pointer-width
    e   | endian
    fe  | features
    c   | cardbox-features
    t   | target
    r   | rust-flags
    u   | uts (only on unix)
"#
  .pipe(eprint)
}

fn target_info_comment() -> Sstr {
  r##"Usage: target /
  OR: target [key1] [optional_key2]

  e.g., `target os`, `target a`, `target fe`"##
}

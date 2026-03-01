use std::{borrow::Cow, fs::exists as fs_exists, io};

use testutils::{
  bool_ext::BoolExt,
  os_cmd::{MiniStr, Runner},
  tap::{Conv, Pipe},
};

mod build_on_debian;
mod linux_musl;
mod zig;

fn run_command<C: AsRef<str>>(command: C) -> io::Result<()> {
  command
    .as_ref()
    .conv::<Runner>()
    .run_command()
}

const fn sstrip_bin() -> &'static str {
  "/usr/bin/sstrip"
}

/// `sstrip /path/to/bin_file`
fn strip_elf_header(file: MiniStr) -> io::Result<()> {
  ensure_elfkickers_installed()?;

  vec![sstrip_bin().into(), file]
    .conv::<Runner>()
    .run_command()
}

/// `apt install elfkickers`
fn ensure_elfkickers_installed() -> io::Result<()> {
  let err =
    "If you're using Arch Linux, try running `pacman -Sy elfkickers` as root";

  if !fs_exists(sstrip_bin())? {
    fs_exists("/usr/bin/apt")?.then_ok_or(io::Error::other(err))?;

    "sudo apt update".pipe(run_command)?;
    "sudo apt install -y elfkickers".pipe(run_command)?
  }

  Ok(())
}

fn remove_text_once<'a>(src: &'a str, text: &str) -> Cow<'a, str> {
  match src.contains(text) {
    true => src.replacen(text, "", 1).into(),
    _ => src.into(),
  }
}

/// e.g., "riscv64a23-unknown-linux-musl" => "riscv64a23-linux-musl"
///
/// ## Example
///
/// ```
/// let text = "riscv64a23-unknown-linux-musl";
/// let result = remove_text_unknown_once(text);
/// assert_eq!(result, "riscv64a23-linux-musl");
///
/// let result2 = remove_text_once(text, "-unknown");
/// assert_eq!(result, result2);
/// ```
fn remove_unknown_text_once(src: &'_ str) -> Cow<'_, str> {
  remove_text_once(src, "-unknown")
}

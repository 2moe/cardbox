/*!
// ==============================
args=(
  -static -Oz
  -nostartfiles
  -Wl,-e,_start
  -Wl,--strip-all
  -Wl,--build-id=none,--nmagic,--no-eh-frame-hdr
  linux-musl.c
)
musl-gcc $args

if !fs::exists("/usr/bin/sstrip")? {
  "sudo apt update" |> run_command;
  "sudo apt install -y elfkickers" |> run_command
}
sstrip a.out

// ==============================
#include <unistd.h>

void _start(void) {
  write(1, "mipsel-unknown-linux-musl\n", 26);
  _exit(0);
}
*/

use std::{
  fs, io, iter,
  path::{Path, PathBuf},
};

use testutils::{
  os_cmd::{MiniStr, Runner, fmt_compact, presets::cargo_build::RustcTarget},
  tap::{Conv, Pipe},
};

use crate::{
  cardbox_target::{remove_text_once, remove_unknown_text_once, strip_elf_header},
  manifest_dir,
};

fn gcc_args() -> [MiniStr; 7] {
  [
    "-static",
    "-Oz",
    "-nostartfiles",
    "-Wl,-e,_start",
    "-Wl,--strip-all",
    "-Wl,--build-id=none,--nmagic,--no-eh-frame-hdr",
    "-o",
  ]
  .map(MiniStr::const_new)
}

fn normalize_gcc_bin(target: &str) -> MiniStr {
  format!("/opt/cross/{target}/bin/{target}-gcc").into()
}

fn generate_c_code<P: AsRef<Path>>(c_file: P, target: &str) -> io::Result<()> {
  let size = target.len() + 1;
  let write_file = |s| fs::write(c_file, s);

  format!(
    r##"
#include <unistd.h>

void _start(void) {{
  write(1, "{target}\n", {size});
  _exit(0);
}}
"##
  )
  .pipe(write_file)
}
// ===================
pub(crate) fn ensure_tmp_c_dir_exists() -> io::Result<PathBuf> {
  let c_dir = manifest_dir().join("tmp/c");
  if !c_dir.exists() {
    fs::create_dir_all(&c_dir)?
  }
  Ok(c_dir)
}

pub(crate) fn init_target_file_path(c_dir: &Path, target: &str) -> MiniStr {
  c_dir
    .parent()
    .expect("Invalid path")
    .join(fmt_compact!("cardbox-target-{target}"))
    .to_string_lossy()
    .into()
}
// ====================================
// <cardbox-target> - <arch> - <os-abi>
fn linux_musl(rs_target: &str, cross_target: Option<&str>) -> io::Result<()> {
  let c_dir = ensure_tmp_c_dir_exists()?;
  let no_unknown_target = match cross_target {
    Some(s @ "s390x-ibm-linux-musl") => remove_text_once(s, "-ibm"),
    _ => remove_unknown_text_once(rs_target),
  };
  let target_file = init_target_file_path(&c_dir, &no_unknown_target);

  let c_file = c_dir.join(fmt_compact!("{no_unknown_target}.c"));

  generate_c_code(&c_file, rs_target)?;

  cross_target
    .unwrap_or(rs_target)
    .pipe(normalize_gcc_bin)
    .pipe(iter::once)
    .chain(gcc_args())
    .chain([target_file.clone()])
    .chain(set_max_page_size(rs_target))
    .chain([c_file.to_string_lossy().into()])
    .collect::<Box<_>>()
    .conv::<Runner>()
    .run_command()?;

  strip_elf_header(target_file)
}

fn set_max_page_size(rs_target: &str) -> Option<MiniStr> {
  use RustcTarget::*;
  [mips_unknown_linux_musl, mipsel_unknown_linux_musl]
    .iter()
    .map(|x| x.as_str())
    .chain(["mips-unknown-linux-muslsf"])
    .any(|x| x.contains(rs_target))
    .then(|| "-Wl,-z,max-page-size=0x1000".into())
}

/// > rs_target == cross_target
fn linux_musl_cross(rs_target: &str) -> io::Result<()> {
  linux_musl(rs_target, None)
}

// ===================
#[ignore]
#[test]
fn build_all_tested() -> io::Result<()> {
  loong64()?;
  s390x()?;
  sh4()?;
  rv32()?;
  // i586()?;
  mips32_le()?;
  microblaze()?;
  microblazeel()?;
  or1k()?;
  ppc()?;
  Ok(())
}

#[ignore]
#[test]
fn loong64() -> io::Result<()> {
  RustcTarget::loongarch64_unknown_linux_musl
    .as_str()
    .pipe(linux_musl_cross)
}

#[ignore]
#[test]
fn s390x() -> io::Result<()> {
  let cross = "s390x-ibm-linux-musl";
  RustcTarget::s390x_unknown_linux_musl
    .as_str()
    .pipe(|rs| linux_musl(rs, cross.into()))
}

#[ignore]
#[test]
fn sh4() -> io::Result<()> {
  let cross = "sh4-multilib-linux-musl";
  linux_musl_cross(cross)
}

#[ignore]
#[test]
fn rv32() -> io::Result<()> {
  let cross = "riscv32-unknown-linux-musl";
  // RustcTarget::riscv32gc_unknown_linux_musl
  //     .as_str()
  //     .pipe(|r| linux_musl(r, cross.into()))
  linux_musl_cross(cross)
}

#[ignore]
#[test]
fn i586() -> io::Result<()> {
  RustcTarget::i586_unknown_linux_musl
    .as_str()
    .pipe(linux_musl_cross)
}

// //  cardbox/t… terminated by signal SIGSEGV (Address boundary error)
// #[ignore]
// #[test]
// fn i686() -> io::Result<()> {
//   RustcTarget::i686_unknown_linux_musl
//     .as_str()
//     .pipe(linux_musl_cross)
// }

/// qemu-microblaze cardbox-target-microblaze-xilinx-linux-musl
#[ignore]
#[test]
fn microblaze() -> io::Result<()> {
  let cross = "microblaze-xilinx-linux-musl";
  linux_musl_cross(cross)
}
#[ignore]
#[test]
fn microblazeel() -> io::Result<()> {
  let cross = "microblazeel-xilinx-linux-musl";
  linux_musl_cross(cross)
}

// // ELF 32-bit MSB executable, MIPS, MIPS-I version 1
// #[ignore]
// #[test]
// fn mips32_be() -> io::Result<()> {
//   RustcTarget::mips_unknown_linux_musl
//     .as_str()
//     .pipe(linux_musl_cross)
// }

// #[ignore]
// #[test]
// fn mips_be_sf() -> io::Result<()> {
//   let cross = "mips-unknown-linux-muslsf";
//   linux_musl_cross(cross)
// }

// #[ignore]
// #[test]
// fn mips64_be() -> io::Result<()> {
//   let cross = "mips64-unknown-linux-musl";

//   RustcTarget::mips64_unknown_linux_muslabi64
//     .as_str()
//     .pipe(|r| linux_musl(r, cross.into()))
// }

// // qemu: uncaught target signal 11
// #[ignore]
// #[test]
// fn mips64_le() -> io::Result<()> {
//   let cross = "mips64el-unknown-linux-musl";

//   RustcTarget::mips64el_unknown_linux_muslabi64
//     .as_str()
//     .pipe(|r| linux_musl(r, cross.into()))
// }

// MIPS, MIPS-I version 1
#[ignore]
#[test]
fn mips32_le() -> io::Result<()> {
  RustcTarget::mipsel_unknown_linux_musl
    .as_str()
    .pipe(linux_musl_cross)
}

#[ignore]
#[test]
fn or1k() -> io::Result<()> {
  let cross = "or1k-unknown-linux-musl";
  linux_musl_cross(cross)
}

#[ignore]
#[test]
fn ppc() -> io::Result<()> {
  use RustcTarget::*;
  [
    powerpc_unknown_linux_musl,
    powerpc64_unknown_linux_musl,
    powerpc64le_unknown_linux_musl,
    // useless: powerpc 32 le
  ]
  .iter()
  .map(|x| x.as_str())
  .try_for_each(linux_musl_cross)
}

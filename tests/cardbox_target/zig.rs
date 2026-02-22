/*!
zig targets | code -
// ==============================
args=(
  cc
  -target riscv64-linux-musl
  -mcpu=baseline_rv64+rva23u64
  -static -Oz
  -flto
  -Wl,--strip-all
  main.c
)
zig $args

if !fs::exists("/usr/bin/sstrip")? {
  "sudo apt update" |> run_command;
  "sudo apt install -y elfkickers" |> run_command
}
sstrip a.out

// ==============================
#include <unistd.h>

int main() {
  write(1, "riscv64a23-unknown-linux-musl\n", 30);
}
*/

use std::{borrow::Cow, env, fs, io, iter, path::Path};

use testutils::{
  os_cmd::{MiniStr, Runner, fmt_compact, presets::TinyVec},
  print_ext::normal::puts,
  tap::{Conv, Pipe, Tap},
};

use crate::cardbox_target::{
  linux_musl::{ensure_tmp_c_dir_exists, init_target_file_path},
  strip_elf_header,
};

type ZigTargetVec = TinyVec<ZigTarget, 3>;

fn zig_cc_args() -> [MiniStr; 5] {
  [
    // "zig",
    "cc",
    "-Oz",
    "-flto",
    "-Wl,--strip-all",
    "-target",
  ]
  .map(MiniStr::const_new)
}

/// - `${XDG_BIN_HOME:-~/.local/bin}/zig`
/// - OR "/usr/local/bin/zig"
fn find_zig_bin() -> MiniStr {
  match env::var("XDG_BIN_HOME") {
    Ok(dir) if !dir.trim().is_empty() => fmt_compact!("{dir}/zig"),
    _ => env::home_dir()
      .map(|x| x.join(".local/bin/zig"))
      .map(|x| x.to_string_lossy().into())
      .unwrap_or("/usr/local/bin/zig".into()),
  }
}

fn generate_main_c_code<P: AsRef<Path>>(c_file: P, target: &str) -> io::Result<()> {
  let size = target.len() + 1;
  let write_file = |s| fs::write(c_file, s);

  format!(
    r##"
#include <unistd.h>
int main() {{
  write(1, "{target}\n", {size});
  return 0;
}}
"##
  )
  .pipe(write_file)
}
// ===================
#[derive(Debug, Default, Clone)]
struct ZigTarget {
  zig_target: MiniStr,
  c_target: Option<MiniStr>,
  mcpu: Option<MiniStr>,
  static_link: Option<bool>,
}

impl<T: AsRef<str>> From<T> for ZigTarget {
  fn from(zig_target: T) -> Self {
    Self {
      zig_target: zig_target
        .as_ref()
        .pipe(remove_suffix_none)
        .into(),
      ..Default::default()
    }
  }
}

impl ZigTarget {
  /// self.static_link:
  ///   - Some(true) => Some("-static"),
  ///   - Some(false) => None,
  ///   - None if zig_target contains "-linux-musl" => Some("-static"),
  ///   - _ => None,
  fn init_static_flag(&self) -> Option<MiniStr> {
    let Self {
      zig_target,
      static_link,
      ..
    } = self;

    let static_flag = MiniStr::const_new("-static");

    match static_link {
      Some(true) => true,
      None if zig_target.contains("-linux-musl") => true,
      _ => false,
    }
    .then_some(static_flag)
  }

  fn build(self) -> io::Result<()> {
    let c_dir = ensure_tmp_c_dir_exists()?;
    let opt_static = self.init_static_flag();

    let Self {
      zig_target,
      c_target,
      mcpu,
      ..
    } = self;

    let target_name = c_target.unwrap_or_else(|| zig_target.clone());
    let target_file = init_target_file_path(&c_dir, &target_name);

    let c_file = c_dir.join(fmt_compact!("{target_name}.c"));
    generate_main_c_code(&c_file, &target_name)?;

    find_zig_bin()
      .pipe(iter::once)
      .chain(zig_cc_args())
      .chain([zig_target])
      .chain(mcpu.map(|x| fmt_compact!("-mcpu={x}")))
      .chain(opt_static)
      .chain(["-o".into(), target_file.clone()])
      .chain([c_file.to_string_lossy().into()])
      .collect::<Box<_>>()
      .conv::<Runner>()
      .run_command()?;

    strip_elf_header(target_file)
  }

  fn with_c_target(mut self, c_target: impl Into<MiniStr>) -> Self {
    self.c_target = into_some_ministr(c_target);
    self
  }

  fn with_mcpu(mut self, mcpu: impl Into<MiniStr>) -> Self {
    self.mcpu = into_some_ministr(mcpu);
    self
  }

  #[allow(dead_code)]
  fn with_static_link(mut self, value: bool) -> Self {
    self.static_link = Some(value);
    self
  }

  /// mcpu: i486 | i586 | i686
  fn update_x86_target(&mut self, mcpu: &str) {
    let Self { zig_target, .. } = &self;
    let prefix = "x86-";
    if !zig_target.starts_with(prefix) {
      return;
    }
    self.c_target = replace_c_target(zig_target, prefix, mcpu);
    self.mcpu = into_some_ministr(mcpu);
  }

  /// mcpu: x86_64_v2 | x86_64_v3 | x86_64_v4 | znver4
  fn update_x64_target(&mut self, mcpu: &str) {
    if mcpu == "x86_64" {
      return;
    }

    let Self { zig_target, .. } = &self;
    let prefix = "x86_64-";
    if !zig_target.starts_with(prefix) {
      return;
    }

    // starts_with "znver4" => "x86_64-znver4"
    self.c_target = match mcpu.starts_with("x86_") {
      false => fmt_compact!("x86_64_{mcpu}"),
      _ => mcpu.into(),
    }
    .pipe(|arch| replace_c_target(zig_target, prefix, &arch));

    self.mcpu = into_some_ministr(mcpu);
  }

  // mcpu: v8_2a | v8_8a | v9a
  fn update_arm64_target(&mut self, mcpu: &str) {
    if mcpu == "aarch64" {
      return;
    }

    let Self { zig_target, .. } = &self;
    let prefix = "aarch64-";
    if !zig_target.starts_with(prefix) {
      return;
    }

    let arch = fmt_compact!("aarch64_{mcpu}");
    self.c_target = replace_c_target(zig_target, prefix, &arch);
    self.mcpu = into_some_ministr(fmt_compact!("generic+{mcpu}"));
  }

  // feat: g | gc | a23
  fn update_rv64_target(&mut self, feat: &str) {
    let Self { zig_target, .. } = &self;
    let prefix = "riscv64-";
    if !zig_target.starts_with(prefix) {
      return;
    }

    let arch = fmt_compact!("riscv64{feat}");
    self.c_target = replace_c_target(zig_target, prefix, &arch);
    let features = match feat {
      "a23" => "baseline_rv64+rva23u64",
      "gc" | "imafdc" => "sifive_u54",
      "g" => "rocket_rv64+m+a+d",
      x => x,
    };
    self.mcpu = into_some_ministr(features);
  }
}

fn replace_c_target(zig_target: &str, prefix: &str, mcpu: &str) -> Option<MiniStr> {
  zig_target
    .replacen(prefix, &fmt_compact!("{mcpu}-"), 1)
    .pipe(into_some_ministr)
}

fn into_some_ministr(s: impl Into<MiniStr>) -> Option<MiniStr> {
  Some(s.into())
}

const fn zig_targets() -> [&'static str; 107] {
  [
    "arc-linux-gnu",
    "arm-freebsd-eabihf",
    "arm-linux-gnueabi",
    "arm-linux-gnueabihf",
    "arm-linux-musleabi",
    "arm-linux-musleabihf",
    "arm-netbsd-eabi",
    "arm-netbsd-eabihf",
    "arm-openbsd-eabi",
    "armeb-linux-gnueabi",
    "armeb-linux-gnueabihf",
    "armeb-linux-musleabi",
    "armeb-linux-musleabihf",
    "armeb-netbsd-eabi",
    "armeb-netbsd-eabihf",
    "thumb-linux-musleabi",
    "thumb-linux-musleabihf",
    "thumb-windows-gnu",
    "thumbeb-linux-musleabi",
    "thumbeb-linux-musleabihf",
    "aarch64-freebsd-none",
    "aarch64-linux-gnu",
    "aarch64-linux-musl",
    "aarch64-maccatalyst-none",
    "aarch64-macos-none",
    "aarch64-netbsd-none",
    "aarch64-openbsd-none",
    "aarch64-windows-gnu",
    "aarch64_be-linux-gnu",
    "aarch64_be-linux-musl",
    "aarch64_be-netbsd-none",
    "csky-linux-gnueabi",
    "csky-linux-gnueabihf",
    "hexagon-linux-musl",
    "loongarch64-linux-gnu",
    "loongarch64-linux-gnusf",
    "loongarch64-linux-musl",
    "loongarch64-linux-muslsf",
    "m68k-linux-gnu",
    "m68k-linux-musl",
    "m68k-netbsd-none",
    "mips-linux-gnueabi",
    "mips-linux-gnueabihf",
    "mips-linux-musleabi",
    "mips-linux-musleabihf",
    "mips-netbsd-eabi",
    "mips-netbsd-eabihf",
    "mipsel-linux-gnueabi",
    "mipsel-linux-gnueabihf",
    "mipsel-linux-musleabi",
    "mipsel-linux-musleabihf",
    "mipsel-netbsd-eabi",
    "mipsel-netbsd-eabihf",
    "mips64-linux-gnuabi64",
    "mips64-linux-gnuabin32",
    "mips64-linux-muslabi64",
    "mips64-linux-muslabin32",
    "mips64-openbsd-none",
    "mips64el-linux-gnuabi64",
    "mips64el-linux-gnuabin32",
    "mips64el-linux-muslabi64",
    "mips64el-linux-muslabin32",
    "mips64el-openbsd-none",
    "powerpc-linux-gnueabi",
    "powerpc-linux-gnueabihf",
    "powerpc-linux-musleabi",
    "powerpc-linux-musleabihf",
    "powerpc-netbsd-eabi",
    "powerpc-netbsd-eabihf",
    "powerpc-openbsd-eabihf",
    "powerpc64-freebsd-none",
    "powerpc64-linux-gnu",
    "powerpc64-linux-musl",
    "powerpc64-openbsd-none",
    "powerpc64le-freebsd-none",
    "powerpc64le-linux-gnu",
    "powerpc64le-linux-musl",
    "riscv32-linux-gnu",
    "riscv32-linux-musl",
    "riscv64-freebsd-none",
    "riscv64-linux-gnu",
    "riscv64-linux-musl",
    "riscv64-openbsd-none",
    "s390x-linux-gnu",
    "s390x-linux-musl",
    "sparc-linux-gnu",
    "sparc-netbsd-none",
    "sparc64-linux-gnu",
    "sparc64-netbsd-none",
    "sparc64-openbsd-none",
    "wasm32-wasi-musl",
    "x86-freebsd-none",
    "x86-linux-gnu",
    "x86-linux-musl",
    "x86-netbsd-none",
    "x86-openbsd-none",
    "x86-windows-gnu",
    "x86_64-freebsd-none",
    "x86_64-linux-gnu",
    "x86_64-linux-gnux32",
    "x86_64-linux-musl",
    "x86_64-linux-muslx32",
    "x86_64-maccatalyst-none",
    "x86_64-macos-none",
    "x86_64-netbsd-none",
    "x86_64-openbsd-none",
    "x86_64-windows-gnu",
  ]
}

fn remove_suffix<'a>(src: &'a str, suffix: &str) -> Cow<'a, str> {
  if !src.ends_with(suffix) {
    return src.into();
  }

  match src.rfind(suffix) {
    Some(pos) => (&src[..pos]).into(),
    _ => src.into(),
  }
}
/// e.g.,
///  - remove_suffix_none("riscv64-openbsd-none") => "riscv64-openbsd"
fn remove_suffix_none(src: &'_ str) -> Cow<'_, str> {
  remove_suffix(src, "-none")
}

#[test]
fn assert_remove_suffix() {
  let text = "riscv64-openbsd-none";
  remove_suffix_none(text) //
    .tap(|x| assert_eq!(x, "riscv64-openbsd"));

  remove_suffix(text, "-none") //
    .tap(|x| assert_eq!(x, "riscv64-openbsd"));

  remove_suffix(text, "-bsd") //
    .tap(|x| assert_eq!(x, text));
}

// ===================
#[ignore]
#[test]
fn show_bsd() {
  zig_targets()
    .iter()
    .filter(|x| x.contains("bsd-"))
    .for_each(puts)
}

/// - x86 => i{4,5,6}86
/// - x86_64 => x86_64_v{1,2,3,4}
fn expand_zig_targets(target: ZigTarget) -> ZigTargetVec {
  match target.zig_target.as_str() {
    z if z.starts_with("x86-") => ["i486", "i586", "i686"]
      .map(|x| {
        target
          .clone()
          .tap_mut(|new| new.update_x86_target(x))
      })
      .into_iter()
      .collect(),
    z if z.starts_with("x86_64-") => {
      ["x86_64", "x86_64_v2", "x86_64_v3", "x86_64_v4", "znver4"]
        .map(|x| {
          target
            .clone()
            .tap_mut(|new| new.update_x64_target(x))
        })
        .into_iter()
        .collect()
    }
    z if z.starts_with("aarch64-") => ["aarch64", "v8_2a", "v8_8a", "v9a"]
      .map(|x| {
        target
          .clone()
          .tap_mut(|new| new.update_arm64_target(x))
      })
      .into_iter()
      .collect(),
    z if z.starts_with("riscv64-") => ["gc", "a23"]
      .map(|x| {
        target
          .clone()
          .tap_mut(|new| new.update_rv64_target(x))
      })
      .into_iter()
      .collect(),
    _ => iter::once(target).collect(),
  }
}

// === OpenBSD ===
#[ignore]
#[test]
fn openbsd() -> io::Result<()> {
  zig_targets()
    .iter()
    .filter(|x| x.contains("-openbsd"))
    .filter(|x| !x.contains("sparc64-"))
    .map(ZigTarget::from)
    .flat_map(expand_zig_targets)
    .try_for_each(|x| x.build())
}

#[ignore]
#[test]
fn netbsd() -> io::Result<()> {
  zig_targets()
    .iter()
    .filter(|x| x.contains("-netbsd"))
    .filter(|x| {
      // sparc64: ld.lld: warning: cannot find entry symbol _start;
      // not setting start address
      !["m68k-", "sparc-", "sparc64-"]
        .iter()
        .any(|arch| x.contains(arch))
    })
    .map(ZigTarget::from)
    .flat_map(expand_zig_targets)
    .try_for_each(|x| x.build())
}

#[ignore]
#[test]
fn freebsd() -> io::Result<()> {
  zig_targets()
    .iter()
    .filter(|x| x.contains("-freebsd"))
    .filter(|x| {
      !["m68k-", "sparc-", "sparc64-", "csky-", "hexagon-"]
        .iter()
        .any(|arch| x.contains(arch))
    })
    .map(ZigTarget::from)
    .flat_map(expand_zig_targets)
    .try_for_each(|x| x.build())
}

#[ignore]
#[test]
fn linux() -> io::Result<()> {
  zig_targets()
    .iter()
    .filter(|x| x.contains("-linux"))
    .filter(|x| {
      [
        "arm-linux-musleabi",
        "arm-linux-musleabihf",
        "armeb-linux-musleabi",
        "armeb-linux-musleabihf",
        // "arc-",
        "thumb-",
        "thumbeb-",
        "aarch64_be-linux-musl",
        "loongarch64-linux-muslsf",
        // "csky-linux-gnueabihf",
        "hexagon",
        // "sparc",
        // "sparc64",
        "mips-linux-musleabi",
        "mips-linux-musleabihf",
        "mipsel-linux-musleabi",
        "mipsel-linux-musleabihf",
        "mips64-linux-muslabi64",
        "mips64el-linux-muslabi64",
        "x86-linux-musl",
      ]
      .iter()
      .any(|arch| x.contains(arch))
    })
    .map(ZigTarget::from)
    .flat_map(expand_zig_targets)
    .try_for_each(|x| x.build())?;
  linux_riscv64_a23u64_musl()?;
  linux_arm64_v8a_and_v9_musl()?;
  Ok(())
}
// #[ignore]
// #[test]
// fn openbsd_loong64() -> io::Result<()> {
//   "loongarch64-openbsd"
//     .conv::<ZigTarget>()
//     .build()
// }

// ===  ===
#[ignore]
#[test]
fn openbsd_arm64() -> io::Result<()> {
  "aarch64-openbsd"
    .conv::<ZigTarget>()
    .build()
}

// ===  ===
#[ignore]
#[test]
fn linux_riscv64_a23u64_musl() -> io::Result<()> {
  "riscv64-linux-musl"
    .conv::<ZigTarget>()
    .with_c_target("riscv64a23-linux-musl")
    .with_mcpu("baseline_rv64+rva23u64")
    .build()
}

/// i486 and i686, no i586
#[ignore]
#[test]
fn linux_x86_musl() -> io::Result<()> {
  for i in [4, 6] {
    "x86-linux-musl"
      .conv::<ZigTarget>()
      .with_c_target(fmt_compact!("i{i}86-linux-musl"))
      .with_mcpu(fmt_compact!("i{i}86"))
      .build()?;
  }
  Ok(())
}

/// MIPS, MIPS32 rel2 version 1
#[ignore]
#[test]
fn linux_mips_hf_musl() -> io::Result<()> {
  "mips-linux-musleabihf"
    .conv::<ZigTarget>()
    .with_c_target("mips32r2-linux-musl")
    .build()
}

#[ignore]
#[test]
fn linux_mips_sf_musl() -> io::Result<()> {
  "mips-linux-musleabi"
    .conv::<ZigTarget>()
    .with_c_target("mips32r2-linux-muslsf")
    .build()
}

// #[ignore]
// #[test]
// fn linux_arm64_v9a_musl() -> io::Result<()> {
//   "aarch64-linux-musl"
//     .conv::<ZigTarget>()
//     .with_c_target("aarch64_v9a-linux-musl")
//     .with_mcpu("generic+v9a")
//     .build()
// }

#[ignore]
#[test]
fn linux_arm64_v8a_and_v9_musl() -> io::Result<()> {
  for feat in [
    // "v8_1a",
    "v8_2a", // "v8_3a", "v8_4a", "v8_5a", "v8_6a",
    // "v8_7a",
    "v8_8a", // "v8_9a",
    "v9a",
  ] {
    "aarch64-linux-musl"
      .conv::<ZigTarget>()
      .with_c_target(fmt_compact!("aarch64_{feat}-linux-musl"))
      .with_mcpu(fmt_compact!("generic+{feat}"))
      .build()?;
  }
  Ok(())
}
#[ignore]
#[test]
fn linux_x64_glibc_2_17_to_2_43() -> io::Result<()> {
  (17..=43)
    .map(|i| fmt_compact!("x86_64-linux-gnu.2.{i}"))
    .map(ZigTarget::from)
    .try_for_each(|x| x.build())
}

#[ignore]
#[test]
fn show_cardbox_target_version() {
  puts(cardbox_target::version());
}

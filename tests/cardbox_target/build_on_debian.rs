//! This file can only run on Debian/Ubuntu.

use std::{fs, io};

use testutils::{
  bool_ext::BoolExt,
  os_cmd::{
    CommandSpawner, MiniStr, fmt_compact,
    presets::{
      CargoCmd,
      cargo_build::{RustcTarget, SubCmd},
    },
  },
  print_ext::normal::edbg,
  tap::{Conv, Pipe, Tap},
};

use crate::{
  cardbox_target::{
    linux_musl::{ensure_tmp_c_dir_exists, init_target_file_path},
    remove_unknown_text_once, run_command, strip_elf_header,
  },
  manifest_dir,
};

fn gnu_linux_rustflags(panic_abort: bool, arch: &str) -> [(MiniStr, MiniStr); 1] {
  // "-Clink-arg=-Wl,-e,_start",
  // "-Clink-arg=-nostartfiles",
  let value = vec![
    "-Ctarget-feature=+crt-static",
    "-Clink-arg=-Wl,--build-id=none,--nmagic,-z,nognustack,--no-eh-frame-hdr",
    "-Crelocation-model=static",
  ]
  .tap_mut(|s| {
    if panic_abort {
      s.extend(["-Zunstable-options", "-Cpanic=immediate-abort"])
    }
    match arch {
      "armv5" | "armv6" | "armv7" => s.push("-Cforce-unwind-tables=no"),
      _ => {}
    }
  })
  .join(" ")
  .into();

  [("RUSTFLAGS".into(), value)]
}

// objcopy --remove-section=.comment
// target/x86_64-unknown-linux-gnu/fat/cardbox-target
// fn remove_elf_comment(gcc_prefix: &str, rs_target: &str) -> io::Result<()> {
//   vec![
//     fmt_compact!("{gcc_prefix}objcopy"),
//     "--remove-section=.comment".into(),
//     fmt_compact!("target/{rs_target}/fat/cardbox-target"),
//   ]
//   .conv::<Runner>()
//   .run_command()
// }

// apt list | perl -F'/' -anE 'say "- " . $F[0] if
//    /^gcc-(?!multi)[^\d+].*?-linux-gnu/'
/// - gcc-aarch64-linux-gnu
/// - gcc-arc-linux-gnu
/// - gcc-arm-linux-gnueabi
/// - gcc-arm-linux-gnueabihf
/// - gcc-powerpc-linux-gnu
/// - gcc-powerpc64-linux-gnu
/// - gcc-powerpc64le-linux-gnu
/// - gcc-riscv64-linux-gnu
/// - gcc-s390x-linux-gnu
/// - gcc-sparc64-linux-gnu
/// - gcc-x86-64-linux-gnu
/// - gcc-x86-64-linux-gnux32
/// - gcc-alpha-linux-gnu
/// - gcc-hppa-linux-gnu
/// - gcc-hppa64-linux-gnu
/// - gcc-m68k-linux-gnu
/// - gcc-sh4-linux-gnu
/// - gcc-mips-linux-gnu
/// - gcc-mips64-linux-gnuabi64
/// - gcc-mips64el-linux-gnuabi64
/// - gcc-mipsel-linux-gnu
/// - gcc-mipsisa32r6-linux-gnu
/// - gcc-mipsisa32r6el-linux-gnu
/// - gcc-mipsisa64r6-linux-gnuabi64
/// - gcc-mipsisa64r6el-linux-gnuabi64
/// - gcc-i686-linux-gnu
/// - gcc-loongarch64-linux-gnu
fn linux_gnu(arch: &str, rs_target: RustcTarget) -> io::Result<()> {
  let (_gcc_prefix, linker) = ensure_debian_cross_gcc(arch)?;
  let rs_target_str = rs_target.as_str();

  let need_build_std = ensure_rustc_target(rs_target_str).is_err();

  // e.g., CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER
  let linker_env_name = format_linker_env_name(rs_target);

  let envs = gnu_linux_rustflags(need_build_std, arch)
    .to_vec()
    .tap_mut(|v| v.extend([(linker_env_name.into(), linker.into())]))
    .into_boxed_slice()
    .tap(|v| eprintln!("env: {v:?}"));

  CargoCmd::default()
    .with_nightly(true)
    .with_target(rs_target.into())
    .with_sub_command(SubCmd::Build)
    .with_pkg("cardbox-target".into())
    .with_profile("fat".into())
    .into_vec()
    .tap_mut(|v| {
      if need_build_std {
        v.extend(
          ["-Zbuild-std=core,panic_abort", "-Zbuild-std-features="].map(Into::into),
        )
      }
    })
    .tap(edbg)
    .conv::<CommandSpawner>()
    .with_envs(envs.into())
    .spawn()?
    .wait()?
    .success()
    .then_ok_or(io::Error::last_os_error())?;

  let pkg_name = "cardbox-target";

  let c_dir = ensure_tmp_c_dir_exists()?;

  let bin_file = {
    let t = remove_unknown_text_once(rs_target_str);
    init_target_file_path(&c_dir, &t)
  };

  manifest_dir()
    .join(fmt_compact!("target/{rs_target_str}/fat/{pkg_name}"))
    .pipe(|f| fs::rename(f, &bin_file))?;

  strip_elf_header(bin_file)
}

fn format_linker_env_name(rs_target: RustcTarget) -> String {
  let snake_target = rs_target
    .as_str()
    .replace('-', "_");

  format!("cargo_target_{snake_target}_linker").to_ascii_uppercase()
}

fn ensure_rustc_target(target: &str) -> io::Result<()> {
  format!("rustup target add {target}").pipe(run_command)
}

// -> io::Result<(gcc_prefix, gcc_target_bin)>
fn ensure_debian_cross_gcc(arch: &str) -> io::Result<(MiniStr, String)> {
  // gcc-arm-linux-gnueabi
  // gcc-arm-linux-gnueabihf
  let (gcc_arch, gcc_abi) = match arch {
    "armv5" => ("arm", "gnueabi"),
    "armv7" => ("arm", "gnueabihf"),
    "x32" => ("x86_64", "gnux32"),
    _ => (arch, "gnu"),
  };

  let gcc_prefix = fmt_compact!("{gcc_arch}-linux-{gcc_abi}-");
  // e.g., /usr/bin/x86_64-linux-gnu-gcc
  let file = format!("/usr/bin/{gcc_prefix}gcc");

  // e.g., gcc-x86-64-linux-gnu
  let deb_pkg = gcc_arch
    .replace('_', "-")
    .pipe(|x| fmt_compact!("gcc-{x}-linux-{gcc_abi}"));

  if !fs::exists(&file)? {
    "sudo apt update".pipe(run_command)?;
    format!("sudo apt install -y {deb_pkg}").pipe(run_command)?
  }

  Ok((gcc_prefix, file))
}
// ===================
#[ignore]
#[test]
fn build_all_tested() -> io::Result<()> {
  x64()?;
  x64p32()?;
  arm64()?;
  riscv64()?;
  armv5te()?;
  armv7a()?;
  Ok(())
}

/**
RUSTFLAGS='
  -C link-arg=-Wl,--build-id=none,--nmagic,-z,nognustack,--no-eh-frame-hdr
  -C relocation-model=static
  -C target-feature=+crt-static
  -C link-arg=-nostartfiles
' cargo build --target x86_64-unknown-linux-gnu --package cardbox-target --profile fat
*/
#[ignore]
#[test]
fn x64() -> io::Result<()> {
  linux_gnu("x86_64", RustcTarget::x86_64_unknown_linux_gnu)
}

/// amd64p32
#[ignore]
#[test]
fn x64p32() -> io::Result<()> {
  linux_gnu("x32", RustcTarget::x86_64_unknown_linux_gnux32)
}

#[ignore]
#[test]
fn arm64() -> io::Result<()> {
  linux_gnu("aarch64", RustcTarget::aarch64_unknown_linux_gnu)
}

#[ignore]
#[test]
fn riscv64() -> io::Result<()> {
  linux_gnu("riscv64", RustcTarget::riscv64gc_unknown_linux_gnu)
}

// #[ignore]
// #[test]
// fn riscv64_a23u64() -> io::Result<()> {
//   linux_gnu("riscv64", RustcTarget::riscv64a23_unknown_linux_gnu)
// }

#[ignore]
#[test]
fn armv5te() -> io::Result<()> {
  linux_gnu("armv5", RustcTarget::armv5te_unknown_linux_gnueabi)
}

#[ignore]
#[test]
fn armv7a() -> io::Result<()> {
  linux_gnu("armv7", RustcTarget::armv7_unknown_linux_gnueabihf)
}

// failed to build rs std
// #[ignore]
// #[test]
// fn mipsle() -> io::Result<()> {
//   linux_gnu("mipsel", RustcTarget::mipsel_unknown_linux_gnu)
// }

// #[ignore]
// #[test]
// fn sparc64() -> io::Result<()> {
//   linux_gnu("sparc64", RustcTarget::sparc64_unknown_linux_gnu)
// }
// ============
// qemu signal 11
// #[ignore]
// #[test]
// fn ppc64le() -> io::Result<()> {
//   linux_gnu("powerpc64le", RustcTarget::powerpc64le_unknown_linux_gnu)
// }

// #[ignore]
// #[test]
// fn ppc() -> io::Result<()> {
//   linux_gnu("powerpc", RustcTarget::powerpc_unknown_linux_gnu)
// }

// #[ignore]
// #[test]
// fn ppc64() -> io::Result<()> {
//   linux_gnu("powerpc64", RustcTarget::powerpc64_unknown_linux_gnu)
// }

// #[ignore]
// #[test]
// fn s390x() -> io::Result<()> {
//   linux_gnu("s390x", RustcTarget::s390x_unknown_linux_gnu)
// }
// #[ignore]
// #[test]
// fn loong64() -> io::Result<()> {
//   linux_gnu("loongarch64", RustcTarget::loongarch64_unknown_linux_gnu)
// }

// #[ignore]
// #[test]
// fn i686() -> io::Result<()> {
//   linux_gnu("i686", RustcTarget::i686_unknown_linux_gnu)
// }

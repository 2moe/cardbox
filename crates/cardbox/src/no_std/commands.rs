#![allow(unused_imports)]

use alloc::{borrow::Cow, string::String};
use core::ffi::CStr;

use cardbox::cli::{self, is_named_bin, trim_exe_suffix, CArgs};
use tinyvec::TinyVec;

use crate::{eprintln, println};

#[allow(dead_code)]
const PKG_VER: &str = env!("CARGO_PKG_VERSION");

type AvaCmds<'a> = TinyVec<[&'a str; 24]>;

#[allow(dead_code)]
mod bin {
  pub(crate) const COPY: [&str; 2] = ["copy", "cp"];
  pub(crate) const LIST: [&str; 2] = ["list", "ls"];
  pub(crate) const PRINT: [&str; 2] = ["print", "echo"];
  pub(crate) const CAT: [&str; 3] = ["cat", "cats", "cat-str"];
  pub(crate) const UNLINK: [&str; 1] = ["unlink"];
  pub(crate) const RMDIR: [&str; 1] = ["rmdir"];
  pub(crate) const SYMLINK: [&str; 1] = ["symlink"];
  pub(crate) const HARDLINK: [&str; 1] = ["hardlink"];
  pub(crate) const MKDIR: [&str; 1] = ["mkdir"];
  pub(crate) const CHMOD: [&str; 2] = ["change-mode", "chmod"];
  pub(crate) const GET_ENV: [&str; 1] = ["get-env"];
  pub(crate) const UTS_NAME: [&str; 1] = ["utsname"];
}

fn get_ava_cmds<'a>() -> AvaCmds<'a> {
  #[allow(unused_mut)]
  let mut cmds = AvaCmds::new();

  #[cfg(feature = "list")]
  cmds.extend(bin::LIST);

  #[cfg(feature = "copy")]
  cmds.extend(bin::COPY);

  #[cfg(feature = "print")]
  cmds.extend(bin::PRINT);

  #[cfg(feature = "cat")]
  cmds.extend(bin::CAT);

  #[cfg(feature = "unlink")]
  cmds.extend(bin::UNLINK);

  #[cfg(feature = "rmdir")]
  cmds.extend(bin::RMDIR);

  #[cfg(feature = "symlink")]
  cmds.extend(bin::SYMLINK);

  #[cfg(feature = "hardlink")]
  cmds.extend(bin::HARDLINK);

  #[cfg(feature = "mkdir")]
  cmds.extend(bin::MKDIR);

  #[cfg(feature = "chmod")]
  cmds.extend(bin::CHMOD);

  #[cfg(feature = "env")]
  cmds.extend(bin::GET_ENV);

  #[cfg(feature = "utsname")]
  cmds.extend(bin::UTS_NAME);

  cmds
}

pub(crate) struct AvaStatus<'a> {
  pub(crate) status: bool,
  pub(crate) cmds: Option<AvaCmds<'a>>,
  pub(crate) args_len: usize,
}

impl<'a> AvaStatus<'a> {
  fn new(status: bool, available: Option<AvaCmds<'a>>, args_len: usize) -> Self {
    Self {
      status,
      cmds: available,
      args_len,
    }
  }
}

pub(crate) fn detect_and_run_available_command(mut args: CArgs) -> AvaStatus {
  let mut exe = args.remove(0).to_string_lossy();

  let cmds = {
    let ava = get_ava_cmds();
    if ava.is_empty() {
      return AvaStatus::new(false, None, 1);
    }
    ava
  };

  exe = trim_exe_suffix(exe);
  #[allow(unused_assignments)]
  if !is_named_bin(&exe, &cmds) {
    if args.is_empty() {
      return AvaStatus::new(false, Some(cmds), 0);
    }
    exe = args.remove(0).to_string_lossy();
    exe = trim_exe_suffix(exe);
  }

  #[allow(unused_variables)]
  let ret = AvaStatus::new(true, None, 0);

  #[cfg(feature = "list")]
  {
    if is_named_bin(&exe, &bin::LIST) {
      run_cmd_list(&args);
      return ret;
    }
  }

  #[cfg(feature = "copy")]
  {
    if is_named_bin(&exe, &bin::COPY) {
      run_cmd_copy(&mut args);
      return ret;
    }
  }

  #[cfg(feature = "print")]
  {
    if is_named_bin(&exe, &bin::PRINT) {
      join_str_with_space_and_print(&args);
      return ret;
    }
  }

  #[cfg(feature = "cat")]
  {
    if is_named_bin(&exe, &bin::CAT) {
      run_cmd_cat(&exe, &mut args).expect("Failed to run cat");
      return ret;
    }
  }

  #[cfg(feature = "unlink")]
  {
    if is_named_bin(&exe, &bin::UNLINK) {
      run_cmd_unlink(&args);
      return ret;
    }
  }

  #[cfg(feature = "rmdir")]
  {
    if is_named_bin(&exe, &bin::RMDIR) {
      run_cmd_rmdir(&args);
      return ret;
    }
  }

  #[cfg(feature = "symlink")]
  {
    if is_named_bin(&exe, &bin::SYMLINK) {
      run_cmd_symlink(&mut args);
      return ret;
    }
  }

  #[cfg(feature = "hardlink")]
  {
    if is_named_bin(&exe, &bin::HARDLINK) {
      run_cmd_hardlink(&mut args);
      return ret;
    }
  }

  #[cfg(feature = "mkdir")]
  {
    if is_named_bin(&exe, &bin::MKDIR) {
      run_cmd_mkdir(&args);
      return ret;
    }
  }

  #[cfg(feature = "chmod")]
  {
    if is_named_bin(&exe, &bin::CHMOD) {
      run_cmd_chmod(&mut args);
      return ret;
    }
  }

  #[cfg(feature = "env")]
  {
    if is_named_bin(&exe, &bin::GET_ENV) {
      run_cmd_get_env(&args);
      return ret;
    }
  }

  #[cfg(feature = "utsname")]
  {
    if is_named_bin(&exe, &bin::UTS_NAME) {
      run_cmd_utsname(&args);
      return ret;
    }
  }

  AvaStatus::new(false, Some(cmds), 1)
}

#[cfg(feature = "utsname")]
fn run_cmd_utsname(args: &[&CStr]) {
  use alloc::format;

  use rustix::path::Arg;

  if args.is_empty() {
    println!(
      "Version: {PKG_VER}\n\
    Usage:
    # Architecture
    ustname arch

    # OS Name
    ustname os
    ustname o

    # Endianness
    ustname endian
    ustname e

    # Pointer width
    ustname ptr
    ustname w

    # CPU features used by the current binary
    ustname feat

    # OS Family (e.g., unix, windows)
    ustname family
    ustname f

    # Machine
    ustname machine
    ustname m

    # kernel name
    utsname sys
    utsname s

    # Kernel release
    utsname rls
    utsname r

    # Kernel version
    utsname ver
    utsname v

    # Network node name
    utsname node
    utsname n

    # all info
    utsname all
    ustname a"
    );
    unsafe { libc::exit(0) }
  }

  type Feats<'f> = TinyVec<[&'f str; 8]>;

  enum UtsValue<'a> {
    Cstr(&'a CStr),
    Feature(Feats<'a>),
    Str(&'a str),
    Other,
  }

  impl<'a> UtsValue<'a> {
    /// Returns `true` if the uts value is [`Other`].
    ///
    /// [`Other`]: UtsValue::Other
    #[must_use]
    fn is_other(&self) -> bool {
      matches!(self, Self::Other)
    }
  }

  impl<'a> From<&'a str> for UtsValue<'a> {
    fn from(v: &'a str) -> Self {
      Self::Str(v)
    }
  }

  impl<'a> From<Feats<'a>> for UtsValue<'a> {
    fn from(v: Feats<'a>) -> Self {
      Self::Feature(v)
    }
  }

  impl<'a> From<&'a CStr> for UtsValue<'a> {
    fn from(v: &'a CStr) -> Self {
      Self::Cstr(v)
    }
  }

  let os_family = env!("__CARDBOX_CFG_FAMILY");
  let cpu_features = env!("__CARDBOX_CFG_FEATURE")
    .split(',')
    .collect::<Feats>();

  let target_os = env!("__CARDBOX_CFG_OS");
  let target_arch = env!("__CARDBOX_CFG_ARCH");
  let pointer_width = env!("__CARDBOX_CFG_POINTER_WIDTH");
  let endian = env!("__CARDBOX_CFG_ENDIAN");

  let uname = rustix::system::uname();
  let machine = uname.machine();
  let sys = uname.sysname();
  let rls = uname.release();
  let ver = uname.version();
  let node = uname.nodename();

  let info = match args[0]
    .to_string_lossy()
    .as_ref()
  {
    "arch" | "architecture" => target_arch.into(),
    "w" | "ptr" | "width" | "ptr-ptr" | "pointer-width" => pointer_width.into(),
    "f" | "family" | "os-family" => os_family.into(),
    "feat" | "feature" | "features" => cpu_features.clone().into(),
    "os" | "o" => target_os.into(),
    "e" | "endian" => endian.into(),
    "m" | "machine" => machine.into(),
    "s" | "sys" | "sysname" | "system" => sys.into(),
    "r" | "rls" | "release" => rls.into(),
    "v" | "ver" | "version" => ver.into(),
    "n" | "node" | "nodename" => node.into(),
    _ => UtsValue::Other,
  };

  if !info.is_other() {
    match info {
      UtsValue::Cstr(c) => println!("{}", c.to_string_lossy()),
      UtsValue::Feature(f) => println!("{f:?}"),
      UtsValue::Str(s) => println!("{s}"),
      _ => {}
    }
    return;
  }

  #[allow(unused_mut)]
  let mut extra_info = match () {
    #[cfg(target_os = "linux")]
    () => String::with_capacity(384),
    #[cfg(not(target_os = "linux"))]
    () => "",
  };

  #[cfg(target_os = "linux")]
  {
    use rustix::system::{sysinfo, Sysinfo};
    extra_info.clear();
    let Sysinfo {
      uptime,
      loads,
      totalram,
      freeram,
      sharedram,
      bufferram,
      totalswap,
      freeswap,
      procs,
      pad,
      totalhigh,
      freehigh,
      mem_unit,
      ..
    } = sysinfo();
    extra_info = format!(
      r##"
    "linux.uptime": {uptime},
    "linux.loads": {loads:?},
    "linux.totalram": {totalram},
    "linux.freeram": {freeram},
    "linux.sharedram": {sharedram},
    "linux.bufferram": {bufferram},
    "linux.totalswap": {totalswap},
    "linux.freeswap": {freeswap},
    "linux.procs": {procs},
    "linux.pad": {pad},
    "linux.totalhigh": {totalhigh},
    "linux.freehigh": {freehigh},
    "linux.mem_unit": {mem_unit},"##
    );
  }

  println!(
    r##"{{
    "arch": {target_arch:?},
    "endian": {endian:?},
    "pointer-width": {pointer_width},
    "os": {target_os:?},{extra_info}
    "os-family": {os_family:?},
    "features": {cpu_features:?},
    "machine": {machine:?},
    "sysname": {sys:?},
    "release": {rls:?},
    "version": {ver:?},
    "nodename": {node:?}
}}"##,
  );
}

#[cfg(feature = "env")]
fn run_cmd_get_env(args: &[&CStr]) {
  use alloc::format;

  let env_len = (0..)
    .find(|&i| (unsafe { *crate::environ.offset(i) }).is_null())
    .unwrap_or(0) as usize;

  let env_ptr_arr = unsafe { alloc::slice::from_raw_parts(crate::environ, env_len) };
  let env_arr = cli::convert_c_args_to_str_arr(env_len, env_ptr_arr);

  let mut map = alloc::collections::BTreeMap::new();

  for (k, v) in env_arr
    .iter()
    .filter_map(|x| x.split_once('='))
  {
    map.insert(k, v);
  }

  let mut s = String::with_capacity(env_len * 128);
  let mut buf = String::with_capacity(256);

  if args.is_empty() {
    s.push('[');

    for (i, (k, v)) in map.iter().enumerate() {
      buf.clear();
      buf = format! {
r##"
    {{
        "index": {i},
        "env": {k:?},
        "value": {v:?}
    }},"##
      };
      s.push_str(&buf);
    }
    // trim ","
    s.pop();
    s.push('\n');
    s.push(']');
    println!("{s}");
    unsafe { libc::exit(0) }
  }

  if args[0] == c"-h" {
    println!(
      "Version: {PKG_VER}\n\
    Usage:
    # Show all environment variables
    get-env

    # Only the env vars for KEY1, KEY2 are displayed
    get-env [KEY1] [KEY2]
    \n\
    Examples:
    get-env path
    get-env xdg_data_home
    get-env term LANG HOME"
    );
    unsafe { libc::exit(0) }
  }

  s.clear();
  s.push('[');
  for query in args
    .iter()
    .map(|x| x.to_string_lossy())
  {
    let value = map.get(query.as_ref());
    buf.clear();
    let json = |query, value: Option<&&str>| {
      format! {
r##"
    {{
        "exists": {},
        "env": {query:?},
        "value": {v:?}
    }},"##, value.is_some(), v = value.unwrap_or(&""),
      }
    };

    buf = json(query.as_ref(), value);
    s.push_str(&buf);

    if value.is_none() {
      buf.clear();
      let query = query.to_ascii_uppercase();
      let value = map.get(query.as_str());
      if value.is_some() {
        buf = json(&query, value);
        s.push_str(&buf);
      }
    }
  }
  // trim ","
  s.pop();
  s.push('\n');
  s.push(']');
  println!("{s}");
}

#[cfg(feature = "unlink")]
fn run_cmd_unlink(args: &[&CStr]) {
  if args.is_empty() {
    println!(
      "Version: {PKG_VER}\n\
        Usage:
    unlink [file]
    unlink [file1] [file2] [file3...]"
    );
    unsafe { libc::exit(0) }
  }

  for &p in args {
    rustix::fs::unlink(p).expect("Failed to unlink (delete) file");
    println!("removed file: {p:?}")
  }
}

#[cfg(feature = "rmdir")]
fn run_cmd_rmdir(args: &[&CStr]) {
  if args.is_empty() {
    println!(
      "Version: {PKG_VER}\n\
    Usage:
    rmdir [empty-dir]
    rmdir [dir1] [dir2] [dir3...]"
    );
    unsafe { libc::exit(0) }
  }

  for &p in args {
    rustix::fs::rmdir(p).expect("[ERROR] Only empty directories can be deleted");
    println!("removed directory: {p:?}")
  }
}

#[cfg(feature = "mkdir")]
fn run_cmd_mkdir(args: &[&CStr]) {
  use cardbox::mkdir::create_dir_all;

  if args.is_empty() {
    println!(
      "Version: {PKG_VER}\n\
    Usage:
    mkdir [dir]
    mkdir [dir1] [dir2...]
    \n\
    Examples:
    mkdir a/b/c/d/e/f/g
    mkdir tmp/a  tmp/b"
    );
    unsafe { libc::exit(0) }
  }
  for p in cli::ignore_flag_args(args) {
    create_dir_all(p);
    println!("created directory: {p:?}")
  }
}

#[cfg(feature = "chmod")]
fn run_cmd_chmod(args: &mut CArgs) {
  use cardbox::chmod;
  use rustix::fs::RawMode;

  if args.len() < 2 {
    println!(
            "Version: {PKG_VER}\n\
    Usage:
    change-mode [mode] [path]
    change-mode [mode] [path1] [path2...]
    \n\
    Examples:
    print 2 > 2.txt
    change-mode 666 2.txt

    print 3 > 3.txt
    change-mode 644 3.txt

    change-mode 777 2.txt 3.txt

    mkdir a b c d
    change-mode 700 a b c d

    ---
    \n\
    Details:

    - Read permission: If set, the file/dir is readable. => 4
    - Write permission: If set, the file/dir can be modified. => 2
    - Execute permission: If set, the file can be executed, the dir can be accessed. => 1

    - R => 4
    - W => 2
    - X => 1

    ---

    - r + x => 4 + 1 = 5
    - r + w => 4 + 2 = 6
    - r + w + x => 4 + 2 + 1 = 7

    ---

    Owner (User), Group, Others

    - 6, 6, 6 => 666 => Everyone: r+w
    - 7, 7, 7 => 777 => Everyone: r+w+x
    - 7, 6, 4 => 764 => Owner: r+w+x, group: r+w, others: r
    - 7, 5, 5 => 755 => Owner: r+w+x, group: r+x, others: r+x
    "
        );
    unsafe { libc::exit(0) }
  }

  let mode = RawMode::from_str_radix(&args.remove(0).to_string_lossy(), 8).expect(
    "Invalid mode, You need to enter the number in octal.
    e.g., 666, 755",
  );

  for p in args.iter() {
    chmod::change_mode(p, mode)
      .expect("Failed to change the mode (file permissions)");
  }
}

#[cfg(feature = "cat")]
fn run_cmd_cat(exe: &str, args: &mut CArgs) -> rustix::io::Result<()> {
  use cardbox::cat::{self, convert_to_u8str_to_stdout, io_copy_to_stdout};

  let mut show_line = false;
  let mut warn_info = false;

  match get_first_arg(args) {
    Some("-h") => cat::show_help_info(),
    None => {
      warn_info = true;
      update_first_arg(args)
    }
    Some("-") => update_first_arg(args),
    Some("-n") => {
      args.remove(0);
      show_line = true;
      if let Some("-") | None = get_first_arg(args) {
        update_first_arg(args)
      }
    }
    _ => {}
  }

  if warn_info {
    eprintln!("[INFO] Run `cats -h` to display help info");
    eprintln!("[WARN] Reading data from stdin");
  }

  let files = cli::ignore_flag_args(args);

  if exe == "cat" {
    io_copy_to_stdout(&files)?;
    return Ok(());
  }

  let files = cli::ignore_flag_args(args);
  convert_to_u8str_to_stdout(&files, show_line)?;
  Ok(())
}

#[cfg(feature = "rustix")]
#[allow(dead_code)]
fn get_first_arg<'a>(args: &'a [&CStr]) -> Option<&'a str> {
  use rustix::path::Arg;

  args
    .first()
    .and_then(|x| x.as_str().ok())
}

#[allow(dead_code)]
fn update_first_arg(args: &mut CArgs) {
  match args.first_mut() {
    Some(f) => *f = c"",
    _ => args.push(c""),
  }
}

#[cfg(feature = "list")]
fn run_cmd_list(args: &[&CStr]) {
  use cardbox::list;
  use rustix::path::Arg;

  if let Some("-h") = args
    .first()
    .and_then(|x| x.as_str().ok())
  {
    list::show_help_info()
  }

  list::list_files(&cli::ignore_flag_args(args)).expect("Failed to list files");
}

#[cfg(feature = "copy")]
fn run_cmd_copy(args: &mut CArgs) {
  use cardbox::copy;

  copy::show_help_info(args.len());
  let dst = get_last_arg(args);

  copy::copy_files(&cli::ignore_flag_args(args), dst).expect("Failed to copy files");
}

#[cfg(any(feature = "copy", feature = "symlink", feature = "hardlink"))]
fn get_last_arg<'b, 'a: 'b>(args: &mut CArgs<'a>) -> &'b CStr {
  args
    .pop()
    .expect("Invalid destination path")
}

#[cfg(feature = "symlink")]
fn run_cmd_symlink(args: &mut CArgs) {
  use cardbox::link;
  link::show_link_help_info(args.len(), "symlink");
  let dst = get_last_arg(args);
  link::new_link(args[0], dst, true).expect("Failed to create symbolic link");
}

#[cfg(feature = "hardlink")]
fn run_cmd_hardlink(args: &mut TinyVec<[&CStr; 8]>) {
  use cardbox::link;
  link::show_link_help_info(args.len(), "hardlink");
  let dst = get_last_arg(args);
  link::new_link(args[0], dst, false).expect("Failed to create hardlink");
}

#[cfg(feature = "print")]
fn join_str_with_space_and_print(args: &[&CStr]) {
  if args.is_empty() {
    println!(
      "Version: {PKG_VER}\n\
    Examples:
    print Hello World

    print 'Hello     World'"
    );

    unsafe { libc::exit(0) }
  }
  let mut s = String::with_capacity(256);
  let collect = |x: Cow<str>| {
    s.push_str(&x);
    s.push(' ');
  };
  args
    .iter()
    .map(|x| x.to_string_lossy())
    .for_each(collect);
  println!("{s}");
}

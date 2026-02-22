use alloc::{format, string::String};
use core::ffi::CStr;

use rustix::{
  fs::{self, Dir, FileType},
  io,
  path::Arg,
};

use crate::{
  common::{readable_unit, CARDBOX_VER},
  fs::{check_and_read_link, open_file, read_link},
  print, println,
};

pub fn show_help_info() {
  println!(
    r#"Version: {CARDBOX_VER}
Usage:
    list [file]
    list [dir]
    list [path1] [path2] [path3...]"#,
  );

  unsafe { libc::exit(0) }
}

/// # Example
///
/// ```
/// #[cfg(unix)] {
///     let files = [c".", c"src", c"/tmp"];
///     let _ = cardbox::list::list_files(&files);
/// }
/// ```
pub fn list_files(files: &[&CStr]) -> io::Result<()> {
  let iter = if files.is_empty() { &[c"."] } else { files };

  for &f in iter {
    let lossy_fname = f.to_string_lossy();
    let file_stat = fs::lstat(f)?;
    {
      let (mode, size) = (file_stat.st_mode, file_stat.st_size);
      let (fsize, unit) = readable_unit(size as _);
      let ftype = FileType::from_raw_mode(mode);

      match ftype {
        FileType::Directory => {}
        FileType::Symlink => {
          let dst = read_link(f);
          print!("{lossy_fname}\t--> {dst:?}");
          if let Some(nested) = check_and_read_link(&dst) {
            print!("  --> {nested:?}")
          }

          println!(
            "\n\t(symlink::size: {fsize} {unit}, type: {ftype:?}, mode: {mode:o}"
          );
          show_detailed_dst_info(&dst);
          continue;
        }
        _ => {
          println!(
            "{lossy_fname}\t(size: {fsize} {unit}, type: {ftype:?}, mode: {mode:o})"
          );
          continue;
        }
      }
    }

    let mut dir = Dir::new(open_file(f)?)?;
    walk_dir(&lossy_fname, &mut dir)?
  }

  Ok(())
}

/// # Example
///
/// ```
/// fn walk() -> rustix::io::Result<()> {
///     use rustix::fs::Dir;
///     use cardbox::{fs::open_file, list::walk_dir};
///
///     let dir_name = c".";
///     let mut dir = Dir::new(open_file(dir_name)?)?;
///     walk_dir(&dir_name.to_string_lossy(), &mut dir)
/// }
/// walk();
/// ```
pub fn walk_dir(lossy_fname: &str, dir: &mut Dir) -> io::Result<()> {
  while let Some(Ok(dir_entry)) = dir.read() {
    if let Ok("." | "..") = dir_entry.file_name().as_str() {
      continue;
    }

    let (ref fname, ftype) = concat_fname_and_get_ftype(lossy_fname, dir_entry);

    let stat = rustix::fs::lstat(fname)?;
    let (mode, size) = (stat.st_mode, stat.st_size);
    let (fsize, unit) = readable_unit(size as _);

    match ftype {
      FileType::Symlink => {
        if let Ok(fname_cstr) = fname.into_c_str() {
          let dst = read_link(&fname_cstr);
          println!("{fname}\t--> {dst:?}")
        }
      }
      _ => println!("{fname}"),
    }
    println!("\t({fsize} {unit}, {ftype:?}, mode: {mode:o})");
  }
  Ok(())
}

/// Suppose there is a **src** directory with the following path structure:
///
/// - src
///   - tests
///   - cli.rs
///   - copy.rs
///   - list.rs
///   - main.rs
///
/// > `lossy_fname` is "src", `sub` is "cli.rs", "copy.rs" ...
///
/// - `concat_fname_and_get_ftype("src", "cli.rs")` returns `("src/cli.rs",
///   RegularFile)`
/// - `concat_fname_and_get_ftype("src", "tests")` returns `("src/cli.rs",
///   Directory)`
pub fn concat_fname_and_get_ftype(
  lossy_fname: &str,
  sub: fs::DirEntry,
) -> (String, FileType) {
  (
    format!(
      "{}/{}",
      lossy_fname.trim_end_matches('/'),
      sub
        .file_name()
        .to_string_lossy()
    ),
    sub.file_type(),
  )
}

fn show_detailed_dst_info(linked: &CStr) {
  if let Ok(stat) = fs::stat(linked) {
    let (dst_mode, dst_size) = (stat.st_mode, stat.st_size);
    let (dst_size, dst_unit) = readable_unit(dst_size as _);
    let dst_type = FileType::from_raw_mode(dst_mode);
    println!("\t(target::size: {dst_size} {dst_unit}, type: {dst_type:?}, mode: {dst_mode:o})");
  }
}

#[cfg(test)]
mod tests {

  #[test]
  fn walk() -> rustix::io::Result<()> {
    use rustix::fs::Dir;

    use crate::{fs::open_file, list::walk_dir};

    let dir_name = c".";
    let mut dir = Dir::new(open_file(dir_name)?)?;
    walk_dir(&dir_name.to_string_lossy(), &mut dir)
  }

  #[test]
  fn ls_files() {
    let files = [c".", c"src", c"/"];
    let _ = crate::list::list_files(&files);
  }
}

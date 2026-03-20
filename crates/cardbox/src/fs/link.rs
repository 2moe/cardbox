#![cfg_attr(target_os = "wasi", feature(wasi_ext))]

use std::{io, path::Path};

use crate::{
  copy::file::{create_dst_parent_dir, validate_and_resolve_dst_path},
  utils::eputs,
};

pub fn link_hard(src_path: &Path, dst_path: &Path) -> Result<(), io::Error> {
  create_dst_parent_dir(dst_path)?;

  let dst_path = validate_and_resolve_dst_path(src_path, dst_path)?;

  remove_existing_file(&dst_path)?;

  std::fs::hard_link(src_path, dst_path)
}

fn remove_existing_file(dst_path: &Path) -> io::Result<()> {
  if dst_path.exists()
    && let Err(e) = std::fs::remove_file(dst_path)
  {
    eputs("[WARN] Failed to remove existing file")?;
    eputs(e.to_string())?;
  }

  Ok(())
}

pub fn link_sym(src_path: &Path, dst_path: &Path) -> Result<(), io::Error> {
  create_dst_parent_dir(dst_path)?;
  let dst_path = validate_and_resolve_dst_path(src_path, dst_path)?;
  if dst_path.exists()
    && let Err(e) = std::fs::remove_file(&dst_path)
  {
    eputs("[WARN] Failed to remove existing file")?;
    eputs(e.to_string())?;
  }
  #[cfg(unix)]
  std::os::unix::fs::symlink(src_path, dst_path)?;
  #[cfg(windows)]
  {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    match src_path {
      p if p.is_dir() => symlink_dir(src_path, dst_path),
      _ => symlink_file(src_path, dst_path),
    }
  }?;
  #[cfg(target_os = "wasi")]
  std::os::wasi::fs::symlink_path(src_path, dst_path)?;
  Ok(())
}

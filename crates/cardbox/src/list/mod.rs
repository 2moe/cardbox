#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
//
use std::{
  io,
  path::{Path, PathBuf},
};

use compact_str::{CompactString as MiniStr, format_compact};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tap::Pipe;

pub mod octal;
#[cfg(unix)]
use crate::list::octal::Mode as OctalMode;

pub mod offset_time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaData {
  pub path: PathBuf,
  pub prop: EntryProperty,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub link: Option<PathBuf>,

  pub size: u64,
  pub size_readable: MiniStr,

  //
  /// permissions
  #[cfg(any(unix, windows))]
  pub perms: Permissions,

  #[cfg(unix)]
  pub nlink: u64,

  #[cfg(unix)]
  pub inode: u64,

  pub time: FileTime,
}

impl MetaData {
  pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    let path = path.as_ref();
    let metadata = path.symlink_metadata()?;

    let size = metadata.len();
    let prop = EntryProperty {
      is_empty: size == 0,
      is_absolute: path.is_absolute(),
      is_dir: path.is_dir(),
      is_file: path.is_file(),
      is_symlink: path.is_symlink(),
    };

    #[cfg(any(unix, windows))]
    let perms = Permissions {
      #[cfg(unix)]
      owner: Owner {
        uid: metadata.uid(),
        gid: metadata.gid(),
      },
      #[cfg(unix)]
      mode: metadata.mode().pipe(OctalMode),
      #[cfg(windows)]
      attrs: metadata.file_attributes(),
    };

    let size_readable = {
      let (num, unit) = crate::utils::readable_unit(size as _);
      format_compact!("{num:.3} {unit}")
    };

    let link = match path.is_symlink() {
      true => path.read_link().ok(),
      _ => None,
    };

    Self {
      path: path.into(),
      prop,
      link,
      size,
      size_readable,

      #[cfg(any(unix, windows))]
      perms,

      #[cfg(unix)]
      nlink: metadata.nlink(),

      #[cfg(unix)]
      inode: metadata.ino(),

      time: init_file_time(&metadata),
    }
    .pipe(Ok)
  }
}

fn init_file_time(metadata: &std::fs::Metadata) -> FileTime {
  let Ok(c_time) = metadata.created() else {
    return Default::default();
  };

  let created = offset_time::systemtime_to_rfc3339(c_time);

  let modified = match metadata.modified() {
    Ok(x) if x == c_time => None,
    Ok(x) => offset_time::systemtime_to_rfc3339(x),
    _ => None,
  };

  // let accessed = match metadata.accessed() {
  //   Ok(x) if x == m_time => None,
  //   Ok(x) => offset_time::systemtime_to_rfc3339(x),
  //   _ => None,
  // };

  FileTime {
    modified,
    created,
    // accessed,
  }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileTime {
  pub created: Option<String>,
  pub modified: Option<String>,
  // pub accessed: Option<String>,
}

/// returns true if the bool is false
pub const fn is_false(b: &bool) -> bool {
  !*b
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
  // readable: bool,
  // writable: bool,
  // executable: bool,
  #[cfg(unix)]
  pub owner: Owner,

  #[cfg(unix)]
  pub mode: OctalMode,

  #[cfg(windows)]
  pub attrs: u32,
}

#[cfg(unix)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
  pub uid: u32,
  pub gid: u32,
}

#[serde_with::apply(
  bool => #[serde(skip_serializing_if = "is_false")],
)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryProperty {
  pub is_empty: bool,
  pub is_absolute: bool,
  pub is_dir: bool,
  pub is_file: bool,
  pub is_symlink: bool,
}

#[cfg(test)]
mod tests {
  use std::{
    env, fs, io,
    path::{Path, PathBuf},
  };

  use serde::Serialize;
  use tap::Pipe;

  use crate::{list::MetaData, utils::puts};
  // use super::*;

  #[ignore]
  #[test]
  fn check_file_metadata() -> io::Result<()> {
    let file = Path::new("../../tmp/cardbox");

    assert! {!file.is_dir()};
    assert! {file.is_symlink()};
    assert! {file.is_file()};

    let metadata = file.symlink_metadata()?;
    dbg!(metadata.is_file());

    if metadata.is_symlink() {
      let real_path = fs::read_link(&file)?;
      println!("{real_path:?}")
    }

    dbg!(metadata.is_symlink());
    let size = metadata.len();
    dbg!(size);

    #[cfg(unix)]
    {
      use std::os::unix::fs::MetadataExt;
      dbg!(metadata.uid(), metadata.gid());
    }

    dbg!(metadata);
    Ok(())
  }

  #[ignore]
  #[test]
  fn show_cargo_toml_metadata_json() -> io::Result<()> {
    MetaData::new("Cargo.toml")?
      .pipe_ref(serde_json::to_string_pretty)?
      .pipe(puts)?;

    Ok(())
  }

  #[ignore]
  #[test]
  fn show_tmp_dir_metadata_json() -> io::Result<()> {
    MetaData::new("tmp")?
      .pipe_ref(serde_json::to_string_pretty)?
      .pipe(puts)?;

    Ok(())
  }

  #[ignore]
  #[test]
  fn show_symlink_metadata_json() -> io::Result<()> {
    MetaData::new("tmp.xx")?
      .pipe_ref(serde_json::to_string_pretty)?
      .pipe(puts)?;
    Ok(())
  }
}

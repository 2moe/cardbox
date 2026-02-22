use alloc::string::String;
use core::ffi::CStr;

use rustix::{
  fs::{mkdir, Mode},
  io::Errno,
};

pub fn create_dir_all(path: &CStr) {
  let mut cur = String::with_capacity(64);

  for component in path
    .to_string_lossy()
    .split(['/', '\\'])
  {
    cur.push_str(component);
    cur.push('/');
    if let Err(e) = mkdir(&cur, Mode::from_raw_mode(0o755)) {
      if e != Errno::EXIST {
        panic!("[ERROR] {cur}: {e}")
      }
    }
  }
}

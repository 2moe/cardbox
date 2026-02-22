use alloc::borrow::Cow;
use core::ffi::CStr;

use tinyvec::TinyVec;

pub type CArgs<'a> = TinyVec<[&'a CStr; 8]>;
type CowArgs<'a> = TinyVec<[Cow<'a, str>; 32]>;

/// Collects C args into `TinyVec<&CStr>`, similar to
/// `std::env::args_os().collect()`
pub fn get_c_args(argc: usize, argv_ptr_arr: &[*const i8]) -> CArgs {
  let mut args = TinyVec::with_capacity(argc + 1);

  unsafe {
    for i in argv_ptr_arr {
      args.push(CStr::from_ptr(*i as _))
    }
  }
  args
}

pub fn convert_c_args_to_str_arr(
  argc: usize,
  argv_ptr_arr: &[*const i8],
) -> CowArgs {
  let mut args = TinyVec::with_capacity(argc + 1);

  unsafe {
    for i in argv_ptr_arr {
      args.push(CStr::from_ptr(*i as _).to_string_lossy())
    }
  }
  args
}

/// Ignores all `-*` arguments.
#[cfg(feature = "rustix")]
pub fn ignore_flag_args<'a>(args: &'a [&'a CStr]) -> TinyVec<[&'a CStr; 8]> {
  args
    .iter()
    .filter(|x| {
      !x.to_string_lossy()
        .starts_with('-')
    })
    .copied()
    .collect()
}

/// Determines if the current file ends with [specified name].
pub fn is_named_bin(exe: &str, names: &[&str]) -> bool {
  let ends = |name| exe.ends_with(name);
  names.iter().any(ends)
}

pub fn trim_exe_suffix(exe: Cow<str>) -> Cow<str> {
  match () {
    #[cfg(not(windows))]
    () => exe,
    #[cfg(windows)]
    () => exe.trim_end_matches(".exe"),
  }
}

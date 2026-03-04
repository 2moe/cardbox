// #![no_std]
// #![no_main]
#![cfg_attr(not(target_os = "wasi"), no_std, no_main)]
// ====
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![cfg_attr(windows, windows_subsystem = "console")]

const MSG: &str = concat!(env!("__cardbox_cfg_target"), "\n");

#[cfg(windows)]
use windows_sys::Win32::{
  Storage::FileSystem::WriteFile,
  System::{
    Console::{GetStdHandle, STD_OUTPUT_HANDLE},
    Threading::ExitProcess,
  },
};

#[cfg(windows)]
#[allow(non_snake_case)]
#[allow(unsafe_op_in_unsafe_fn)]
#[unsafe(no_mangle)]
unsafe extern "C" fn mainCRTStartup() -> ! {
  let console = GetStdHandle(STD_OUTPUT_HANDLE);
  let mut written = 0;

  let status = WriteFile(
    console,
    MSG.as_ptr() as _,
    MSG.len() as _,
    &mut written,
    core::ptr::null_mut(),
  );
  let failure = 0;
  if status != failure && written == MSG.len() as _ {
    ExitProcess(0)
  }
  ExitProcess(2)
}

#[cfg(not(target_os = "wasi"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
  core::intrinsics::abort()
}

#[cfg(target_os = "linux")]
#[cfg(any(
  target_arch = "aarch64",
  target_arch = "arm",
  target_arch = "riscv64",
  target_arch = "x86_64",
))]
#[unsafe(no_mangle)]
extern "C" fn _start() {
  use syscalls::{Sysno, syscall1};
  let out = unsafe { rustix::stdio::take_stdout() };
  let _ = rustix::io::write(&out, MSG.as_bytes());
  let _ = unsafe { syscall1(Sysno::exit, 0) };
}

#[cfg(target_os = "linux")]
#[cfg(not(any(
  target_arch = "aarch64",
  target_arch = "arm",
  target_arch = "riscv64",
  target_arch = "x86_64",
)))]
#[unsafe(no_mangle)]
extern "C" fn _start() -> ! {
  use libc::{_exit, STDOUT_FILENO, write};

  const BUF: &[u8] = MSG.as_bytes();

  unsafe {
    write(STDOUT_FILENO, BUF.as_ptr() as _, BUF.len());
    _exit(0)
  }
}

#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
#[cfg(not(target_os = "linux"))]
#[unsafe(no_mangle)]
extern "C" fn main() -> core::ffi::c_int {
  use libc::{STDOUT_FILENO, write};

  const BUF: &[u8] = MSG.as_bytes();

  unsafe { write(STDOUT_FILENO, BUF.as_ptr() as _, BUF.len()) };
  0
}

/// build command:
///   RUSTFLAGS='-Zunstable-options -Cpanic=immediate-abort'
///   cargo b --profile fat --target=wasm32-wasip1 -Zbuild-std=std,panic_abort
///
/// wasm file size (rustc 1.95.0-nightly 2026-02-21):
///   - wasip1: 3800B (3.8K)
///   - wasip2: 24869B  (25K)
#[cfg(target_os = "wasi")]
fn main() {
  let out = unsafe { rustix::stdio::take_stdout() };
  let _ = rustix::io::write(&out, MSG.as_bytes());
}

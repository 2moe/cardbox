#![allow(unused)]

pub(crate) const CARDBOX_VER: &str = env!("CARGO_PKG_VERSION");

pub fn readable_unit(bytes: i64) -> (i64, &'static str) {
  // It's interesting to note that using the f32/f64 types here results in a 20K
  // larger binary file.
  ["B", "KiB", "MiB", "GiB", "TiB", "PiB"]
    .iter()
    .enumerate()
    .map(|(i, &unit)| (bytes / 1024_i64.pow(i as _), unit))
    .take_while(|(size, _)| *size >= 1)
    .last()
    .unwrap_or((bytes, "B"))
}

pub(crate) const fn yes() -> bool {
  true
}

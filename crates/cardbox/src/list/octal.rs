use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tap::Pipe;
use tinystr::TinyAsciiStr;
/// e.g., "0o100644"
pub type OctModeStr = TinyAsciiStr<8>;

#[derive(Debug, Copy, Clone)]
pub struct Mode(pub u32);

impl Serialize for Mode {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    format_args!("0o{:04o}", self.0).serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Mode {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let to_u32 = |s| u32::from_str_radix(s, 8);

    OctModeStr::deserialize(deserializer)?
      .pipe_deref(to_u32)
      .map_err(serde::de::Error::custom)?
      .pipe(Mode)
      .pipe(Ok)
  }
}

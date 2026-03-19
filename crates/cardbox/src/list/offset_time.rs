use std::{
  sync::OnceLock,
  time::{SystemTime, UNIX_EPOCH},
};

use tap::Pipe;
use time::{
  OffsetDateTime, UtcDateTime, UtcOffset, format_description::well_known::Rfc3339,
};

pub fn systime_to_rfc3339(st: SystemTime) -> Option<String> {
  st.duration_since(UNIX_EPOCH)
    .ok()? // .expect("time went backwards")
    .as_secs()
    .pipe(|x| UtcDateTime::from_unix_timestamp(x as _))
    .ok()? // utc
    .to_offset(*offset_cache())
    .format(&Rfc3339)
    .ok()
}

pub fn offset_cache() -> &'static UtcOffset {
  static T: OnceLock<UtcOffset> = OnceLock::new();

  T.get_or_init(|| {
    let now = OffsetDateTime::now_utc();
    UtcOffset::local_offset_at(now).unwrap_or(UtcOffset::UTC)
  })
}

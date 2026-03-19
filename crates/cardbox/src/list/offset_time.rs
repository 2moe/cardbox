use std::time::{SystemTime, UNIX_EPOCH};

use tap::Pipe;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn systemtime_to_rfc3339(st: SystemTime) -> Option<String> {
  st.duration_since(UNIX_EPOCH)
    .expect("time went backwards")
    .as_secs()
    .pipe(|x| OffsetDateTime::from_unix_timestamp(x as _))
    .ok()?
    .format(&Rfc3339)
    .ok()
}

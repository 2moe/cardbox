use std::io;

use serde_json::Value as JsonValue;

#[cfg(unix)]
#[cfg(feature = "serde")]
pub(crate) fn uts_name() -> JsonValue {
  let uname = cardbox::no_std::uname();

  let machine = uname.machine();
  let sys = uname.sysname();
  let rls = uname.release();
  let ver = uname.version();
  let node = uname.nodename();

  serde_json::json!({
    "machine": machine,
    "sys_name": sys,
    "release": rls,
    "version": ver,
    "node_name": node,
  })
}

pub(crate) fn target_info(args: Option<&[String]>) -> io::Result<()> {
  use cardbox::consts::*;
  //
  Ok(())
}

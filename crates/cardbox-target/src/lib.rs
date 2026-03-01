pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub const fn target() -> &'static str {
  env!("__cardbox_cfg_target")
}

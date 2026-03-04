/// cargo pkg version
pub const fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub const fn target_family() -> &'static str {
  env!("__cardbox_cfg_family")
}
pub const fn target_os() -> &'static str {
  env!("__cardbox_cfg_os")
}
pub const fn target_arch() -> &'static str {
  env!("__cardbox_cfg_arch")
}
pub const fn target_pointer_width() -> &'static str {
  env!("__cardbox_cfg_pointer_width")
}
pub const fn target_endian() -> &'static str {
  env!("__cardbox_cfg_endian")
}
pub const fn target_feature() -> &'static str {
  env!("__cardbox_cfg_feature")
}
pub const fn target() -> &'static str {
  env!("__cardbox_cfg_target")
}

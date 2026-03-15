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
pub const fn target_vendor() -> &'static str {
  env!("__cardbox_cfg_vendor")
}
pub const fn target_env() -> &'static str {
  env!("__cardbox_cfg_env")
}
pub const fn target_abi() -> &'static str {
  env!("__cardbox_cfg_abi")
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
pub const fn cargo_feature() -> &'static str {
  env!("__cardbox_cfg_cargo_feature")
}
pub const fn target() -> &'static str {
  env!("__cardbox_cfg_target")
}
/// separated by 0x1f
pub const fn encoded_rust_flags() -> &'static str {
  env!("__cardbox_cfg_encoded_rust_flags")
}

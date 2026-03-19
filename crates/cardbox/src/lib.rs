#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
# CardBox
*/

pub mod consts;
pub mod utils;

pub mod copy;
pub mod fs;

#[cfg(feature = "list")]
pub mod list;

pub mod path;

// === UNIX only ===
#[cfg(unix)]
#[cfg(feature = "uts_name")]
pub use rustix::system::uname;

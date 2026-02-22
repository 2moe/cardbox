#![no_std]
// cargo +nightly rustdoc --open --all-features -- --cfg __doc
// --document-private-items
#![cfg_attr(__doc, feature(doc_auto_cfg, doc_notable_trait))]

extern crate alloc;

/// copy files
#[cfg(feature = "copy")]
pub mod copy;

#[cfg(feature = "list")]
pub mod list;

/// filesystem I/O api
#[cfg(feature = "fs")]
pub mod fs;

/// command line
#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cat")]
pub mod cat;

/// symlink & hardlink
#[cfg(any(feature = "hardlink", feature = "symlink"))]
pub mod link;

#[cfg(feature = "mkdir")]
pub mod mkdir;

#[cfg(feature = "chmod")]
pub mod chmod;

pub(crate) mod common;

#[allow(unused_imports)]
pub(crate) use libc_print::std_name::{eprintln, print, println};

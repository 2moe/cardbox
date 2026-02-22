#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
# CardBox
*/

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod no_std;

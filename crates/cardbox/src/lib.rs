#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*!
# CardBox
*/

// #[deprecated]
#[cfg(not(windows))]
#[cfg(feature = "rustix")]
pub mod no_std;

#[cfg(feature = "std")]
pub mod imp_std;

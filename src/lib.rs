#![feature(bindings_after_at)]
#![recursion_limit = "10"]

//! Domain specific compression library.
//!
//! # Goals
//!
//! * Compress structured data (e.g., JSON)
//! * Compress better then gzip for specific data formats
//! * Avoid the need for metadata whenever possible
//!   * Instead, a static, predetermined schema may be used to aid in compression and
//!     decompression

#[macro_use]
extern crate smallvec;

pub mod co;
pub mod compress;
pub mod core;
pub mod schema;
pub mod transcode;

/// A collection of commonly used types.
pub mod prelude {
  pub use crate::co::{Block, CompressedObject, Glob, Marker};
  pub use crate::schema::{EnumMode, Schema, Type};
}

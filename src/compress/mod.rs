//! Various compressor implementations.

use crate::glob::Glob;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod enumeration;
mod numerical;
mod uuid;

pub use enumeration::EnumCompressor;

/// Compression result type.
///
/// Since each compressor variant will have different error types we use a
/// dynamic error type for the result instead of a enumeration.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Common trait implemented by all compressors.
///
/// Provides byte-level compression delegating the task of ensuring those bytes
/// are valid for the specific compression medium to the implementor.
///
/// Compressors are required to uphold the following invariant for all values
/// of `x` which produce an `Ok` result:
///
/// ```text
/// decompress(compress(x)) == x
/// ```
pub trait Compressor {
  /// Compresses a sequence of bytes into a glob.
  fn compress(&self, input: &[u8]) -> Result<Glob>;

  /// Decompresses a glob into a sequence of bytes.
  fn decompress(&self, glob: Glob) -> Result<Vec<u8>>;
}

/// A compressor which encodes a sequence of bytes as-is with no modification.
pub struct PassThroughCompressor;

impl Compressor for PassThroughCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    Ok(Glob::new(input.len() * 8, input.to_vec()))
  }

  fn decompress(&self, glob: Glob) -> Result<Vec<u8>> {
    if glob.width % 8 != 0 {
      return Err(PassThroughError::WrongGlobWidth(glob.width).into());
    }

    Ok(glob.data)
  }
}

#[derive(Debug)]
enum PassThroughError {
  WrongGlobWidth(usize),
}

impl Display for PassThroughError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    use PassThroughError::*;
    match self {
      WrongGlobWidth(size) => write!(
        f,
        "wrong glob width for pass through: expected multiple of 8, got {}",
        size
      ),
    }
  }
}

impl Error for PassThroughError {}

/// Returns the builtin compressor with a given `name`.
pub fn builtin(name: &str) -> Option<Box<dyn Compressor>> {
  if name == "uuid" {
    return Some(Box::new(uuid::UuidCompressor));
  }

  // Fixed width integer values are named via the convention "u<num>" where
  // "<num>" is the bit-width of the encoded integer value.
  if name.starts_with('u') {
    let num = &name[1..];
    if num.chars().all(|c| c.is_ascii_digit()) {
      let width = num.parse::<usize>().expect("failed to parse");
      return Some(Box::new(numerical::IntegerCompressor::new(width)));
    }
  }

  // If unable to match `name` then return `None`.
  None
}

//! Compressors for numerical values.

use super::{Compressor, Glob, Result};
use crate::core::math;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A compressor which treats it's input as the little endian representation of
/// a 64-bit integer. The compressed output is the same little endian
/// representation truncated to a specific bit width. Decompressing produces the
/// little endian representation of a 64-bit integer.
pub struct IntegerCompressor {
  width: usize,
}

impl IntegerCompressor {
  /// Constructs a new integer compressor which encodes values as fixed with
  /// integers with a given `width`.
  pub fn new(width: usize) -> Self {
    IntegerCompressor { width }
  }
}

impl Compressor for IntegerCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    // Determine how many bytes should be in the compressed output.
    let byte_count = math::div_ceil(self.width, 8);

    // Truncate or zero-extend `input` to match the required length.
    let mut bytes = if input.len() >= byte_count {
      Vec::from(&input[0..byte_count])
    } else {
      let mut vec = Vec::from(input);
      while vec.len() < byte_count {
        vec.push(0);
      }
      vec
    };

    // Mask off any unused bits in the last byte if the required bit width is
    // not a multiple of 8.
    let valid_trailing_bits = self.width % 8;
    if valid_trailing_bits != 0 {
      let mask = math::low_mask::<u8>(valid_trailing_bits);
      *bytes.last_mut().unwrap() &= mask;
    }

    Ok(Glob::new(self.width, bytes))
  }

  fn decompress(&self, glob: Glob) -> Result<Vec<u8>> {
    let mut bytes = glob.data;
    // If the data is larger than a u64, return an error.
    if bytes.len() >= 8 {
      return Err(IntegerCompressorError::GlobTooLarge(glob.width).into());
    }

    // Zero extend `bytes` so it can be converted to a u64.
    while bytes.len() < 8 {
      bytes.push(0);
    }

    Ok(bytes)
  }
}

#[derive(Debug)]
enum IntegerCompressorError {
  GlobTooLarge(usize),
}

impl Display for IntegerCompressorError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    use IntegerCompressorError::*;
    match self {
      GlobTooLarge(width) => write!(f, "glob too large: {} bits", width),
    }
  }
}

impl Error for IntegerCompressorError {}

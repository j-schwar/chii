//! Encoder for boolean values.

use super::{Compressor, Glob, Result};

/// An encoder for boolean values.
///
/// Expects a single byte which is either 1 for true or 0 false as input. This
/// encoder simply converts that single byte into a 1-bit glob.
pub struct BoolCompressor;

impl Compressor for BoolCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    assert_eq!(input.len(), 1);
    if input[0] == 1 {
      Ok(Glob::new(1, vec![1]))
    } else if input[0] == 0 {
      Ok(Glob::new(1, vec![0]))
    } else {
      panic!("illegal input: {:?}", input);
    }
  }

  fn decompress(&self, glob: Glob) -> Result<Vec<u8>> {
    assert_eq!(glob.width, 1);
    assert!(glob.data == vec![1] || glob.data == vec![0]);
    Ok(glob.data)
  }
}

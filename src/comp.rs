//! The `comp` module defines the foundation of the compression framework along
//! with various general purpose compression implementations.

use crate::bit::BitVecExt;
use crate::math;
use bit_vec::BitVec;
use std::error::Error;

/// The result type returned by the various compressors.
///
/// Each compressor is free to use their own error types.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// The trait implement by all compressors.
///
/// Ideally, `decompress` should be the inverse function of `comp` meaning
/// that `decompress(comp(x)) == x` for all valid x. However, this
/// functionality may not always be desirable. For example, one could wish to
/// encode enumeration variants in a case-insensitive manor.
pub trait Compressor {
  /// Compresses a slice of bytes into a bit vector.
  ///
  /// The implementor may assume that all bytes are a part of the value being
  /// encoded and need not worry about extracting the value from surrounding
  /// context.
  fn compress(&self, bytes: &[u8]) -> Result<BitVec>;

  /// Decompresses a bit vector into a sequence of bytes.
  ///
  /// The implementor may assume that all bits are a part of the value being
  /// decoded and need not worry about extracting the value from surrounding
  /// context.
  fn decompress(&self, bits: BitVec) -> Result<Vec<u8>>;
}

pub fn lookup(name: &str) -> Option<Box<dyn Compressor>> {
  match name {
    "bool" => Some(Box::new(BooleanCompressor)),
    _ => None,
  }
}

/// The identity compressor doesn't perform any compression and instead passes
/// along any input data unmodified.
pub struct IdentityCompressor;

impl Compressor for IdentityCompressor {
  fn compress(&self, bytes: &[u8]) -> Result<BitVec> {
    Ok(BitVec::from_bytes(bytes))
  }

  fn decompress(&self, bits: BitVec<u32>) -> Result<Vec<u8>> {
    if bits.len() % 8 != 0 {
      return Err("encoded data has invalid length".into());
    }
    Ok(bits.to_bytes())
  }
}

/// A compressor for boolean types.
///
/// Compresses a boolean's string representation into a single bit.
struct BooleanCompressor;

impl Compressor for BooleanCompressor {
  fn compress(&self, bytes: &[u8]) -> Result<BitVec> {
    match bytes {
      _ if bytes == b"true" => Ok(BitVec::from_elem(1, true)),
      _ if bytes == b"false" => Ok(BitVec::from_elem(1, false)),
      _ => Err(r#"unexpected source data, expected "true" or "false""#.into()),
    }
  }

  fn decompress(&self, bits: BitVec<u32>) -> Result<Vec<u8>> {
    if bits.len() != 1 {
      return Err("expected a single bit".into());
    }

    let result = if bits[0] { "true" } else { "false" }.as_bytes().to_vec();
    Ok(result)
  }
}

/// Compressor for enumerations of string variants.
///
/// Takes a fixed set of variants and compresses them into unique integer values
/// represented using the minimum required number of bits.
struct EnumCompressor {
  pub variants: Vec<String>,
}

impl Compressor for EnumCompressor {
  fn compress(&self, bytes: &[u8]) -> Result<BitVec> {
    let index = self
      .variants
      .iter()
      .position(|v| v.as_bytes() == bytes)
      .ok_or("unknown enum variant")? as u64;
    let width = math::required_bit_width(self.variants.len());
    let mut bits = BitVec::from_rev_be(index);
    bits.truncate(width);
    Ok(bits)
  }

  fn decompress(&self, mut bits: BitVec<u32>) -> Result<Vec<u8>> {
    bits.zext_or_trunc(64);
    // This can't fail as we just extended the vector to 64 bits
    let index = bits.to_rev_be::<u64>().unwrap();
    let variant: &String = self
      .variants
      .get(index as usize)
      .ok_or("cannot match encoded value to variant")?;
    Ok(variant.as_bytes().to_vec())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn compress_boolean_true() -> Result<()> {
    let bits = BooleanCompressor.compress("true".as_bytes())?;
    assert_eq!(bits, BitVec::from_elem(1, true));
    Ok(())
  }

  #[test]
  fn compress_boolean_false() -> Result<()> {
    let bits = BooleanCompressor.compress("false".as_bytes())?;
    assert_eq!(bits, BitVec::from_elem(1, false));
    Ok(())
  }
}

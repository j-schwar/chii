//! A configurable enumeration compressor.

use super::{Compressor, Glob, Result};
use crate::core::{integer, math};
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A specialized compressor which encodes an enumeration of byte patterns as
/// ordinals. The width of the resultant glob is the minimum number of bits
/// needed to represent all ordinal values.
pub struct EnumCompressor {
  variants: Vec<BytePattern>,
}

impl EnumCompressor {
  /// Constructs a new enumeration compressor for a given set of byte patterns
  /// which represent valid enum variants.
  pub fn from_variants(variants: Vec<BytePattern>) -> Self {
    EnumCompressor { variants }
  }

  /// Constructs a new enumeration compressor from a given set of strings which
  /// define the valid enum variants.
  pub fn from_string_variants<S: AsRef<str>>(variants: &[S]) -> Self {
    EnumCompressor {
      variants: variants
        .iter()
        .map(|s| s.as_ref().bytes().collect())
        .collect(),
    }
  }

  /// The width of the resultant glob in number of bits.
  pub fn result_width(&self) -> usize {
    math::required_bit_width(self.variants.len())
  }
}

impl Compressor for EnumCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    let index = self.variants.iter().position(|v| {
      if v.len() != input.len() {
        return false;
      }
      v.as_slice() == input
    });

    let value = index.ok_or(EnumError::NoMatchingVariant)?;
    let bytes = value.to_le_bytes().to_vec();
    Ok(Glob::new(self.result_width(), bytes))
  }

  fn decompress(&self, mut glob: Glob) -> Result<Vec<u8>> {
    if self.result_width() != glob.width {
      return Err(
        EnumError::GlobLengthMismatch(self.result_width(), glob.width).into(),
      );
    }

    // Pad the glob's data so we can convert it to a u64.
    integer::pad_with_zero(8, &mut glob.data);
    let index = u64::from_le_bytes(glob.data.as_slice().try_into()?) as usize;
    self
      .variants
      .get(index)
      .cloned()
      .ok_or_else(|| EnumError::NoVariantWithIndex(index).into())
  }
}

type BytePattern = Vec<u8>;

#[derive(Debug)]
enum EnumError {
  NoMatchingVariant,
  GlobLengthMismatch(usize, usize),
  NoVariantWithIndex(usize),
}

impl Display for EnumError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    use EnumError::*;
    match self {
      NoMatchingVariant => write!(f, "no matching enum variant"),
      GlobLengthMismatch(expected, actual) => write!(
        f,
        "glob length mismatch, expected {}, found {}",
        expected, actual
      ),
      NoVariantWithIndex(i) => write!(f, "no variant with index: {}", i),
    }
  }
}

impl Error for EnumError {}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn ternary_enum_compress() -> Result<()> {
    let variants = vec!["Foo", "Bar", "Hello World"];
    let compressor = EnumCompressor::from_string_variants(&variants);

    let glob = compressor.compress(b"Foo")?;
    assert_eq!(Glob::new(2, vec![0b00]), glob);

    let glob = compressor.compress(b"Bar")?;
    assert_eq!(Glob::new(2, vec![0b01]), glob);

    let glob = compressor.compress(b"Hello World")?;
    assert_eq!(Glob::new(2, vec![0b10]), glob);

    Ok(())
  }

  #[test]
  fn ternary_enum_decompress() -> Result<()> {
    let variants = vec!["Foo", "Bar", "Hello World"];
    let compressor = EnumCompressor::from_string_variants(&variants);

    let glob = Glob::new(2, vec![0b00]);
    assert_eq!(b"Foo".to_vec(), compressor.decompress(glob)?);

    let glob = Glob::new(2, vec![0b01]);
    assert_eq!(b"Bar".to_vec(), compressor.decompress(glob)?);

    let glob = Glob::new(2, vec![0b10]);
    assert_eq!(b"Hello World".to_vec(), compressor.decompress(glob)?);
    Ok(())
  }
}

//! The `comp` module defines the foundation of the compression framework along
//! with various general purpose compression implementations.

use crate::bit::BitVecExt;
use crate::math;
use anyhow::{anyhow, bail, Error, Result};
use bit_vec::BitVec;

/// Represents a primitive data value to be compressed.
#[derive(Debug, PartialEq)]
pub enum Value {
  Bool(bool),
  Int(i64),
  UInt(u64),
  Float(f64),
  Str(String),
}

impl Value {
  /// A textual description of the variant type; used for error messages.
  fn typename(&self) -> &'static str {
    use Value::*;

    match self {
      Bool(_) => "bool",
      Int(_) | UInt(_) => "int",
      Float(_) => "float",
      Str(_) => "string",
    }
  }
}

fn unexpected_type(value: Value, hint: &'static str) -> Error {
  anyhow!(
    "unexpected value type: {}, expected {}",
    value.typename(),
    hint
  )
}

/// The trait implement by all compressors.
///
/// Ideally, `decompress` should be the inverse function of `comp` meaning
/// that `decompress(comp(x)) == x` for all valid x. However, this
/// functionality may not always be desirable. For example, one could wish to
/// encode enumeration variants in a case-insensitive manor.
pub trait Compressor {
  /// Compresses a value into a sequence of bits.
  fn compress(&self, value: Value) -> Result<BitVec>;

  /// Interprets a sequence of bits as a value.
  fn decompress(&self, bits: BitVec) -> Result<Value>;
}

/// The identity compressor doesn't perform any compression and instead passes
/// along any input data unmodified. It only accepts string values.
pub struct IdentityCompressor;

impl Compressor for IdentityCompressor {
  fn compress(&self, value: Value) -> Result<BitVec> {
    match value {
      Value::Str(s) => Ok(BitVec::from_bytes(s.as_bytes())),
      _ => Err(unexpected_type(value, "string")),
    }
  }

  fn decompress(&self, bits: BitVec) -> Result<Value> {
    if bits.len() % 8 != 0 {
      bail!("unable to convert bit sequence to bytes");
    }
    let bytes = bits.to_bytes();
    let s = String::from_utf8(bytes)?;
    Ok(Value::Str(s))
  }
}

/// A compressor for boolean types.
///
/// Compresses a boolean's string representation into a single bit.
struct BooleanCompressor;

impl Compressor for BooleanCompressor {
  fn compress(&self, value: Value) -> Result<BitVec> {
    match value {
      Value::Bool(b) => Ok(BitVec::from_elem(1, b)),
      _ => Err(unexpected_type(value, "bool")),
    }
  }

  fn decompress(&self, bits: BitVec<u32>) -> Result<Value> {
    if bits.len() != 1 {
      bail!("invalid bit sequence length");
    }

    Ok(Value::Bool(bits[0]))
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
  fn compress(&self, value: Value) -> Result<BitVec> {
    let s = if let Value::Str(s) = value {
      s
    } else {
      return Err(unexpected_type(value, "string"));
    };

    let bytes = s.as_bytes();
    let index = self
      .variants
      .iter()
      .position(|v| v.as_bytes() == bytes)
      .ok_or_else(|| anyhow!("cannot convert {} to enum variant", s))?
      as u64;
    let width = math::required_bit_width(self.variants.len());
    let mut bits = BitVec::from_rev_be(index);
    bits.truncate(width);
    Ok(bits)
  }

  fn decompress(&self, mut bits: BitVec) -> Result<Value> {
    bits.zext_or_trunc(64);
    // This can't fail as we just extended the vector to 64 bits
    let index = bits.to_rev_be::<u64>().unwrap();
    let variant: &String = self
      .variants
      .get(index as usize)
      .ok_or_else(|| anyhow!("cannot match encoded value to variant"))?;
    Ok(Value::Str(variant.clone()))
  }
}

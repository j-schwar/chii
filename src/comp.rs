//! The `comp` module defines the foundation of the compression framework along
//! with various general purpose compression implementations.

use crate::bit::BitVecExt;
use crate::math;
use anyhow::{anyhow, bail, Error, Result};
use bit_vec::BitVec;
use std::convert::TryFrom;

mod boolean;
mod enumeration;
mod identity;

pub use boolean::BooleanCompressor;
pub use enumeration::EnumCompressor;
pub use identity::IdentityCompressor;

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

impl<'a> TryFrom<&'a serde_json::Value> for Value {
  type Error = anyhow::Error;

  fn try_from(v: &'a serde_json::Value) -> Result<Self> {
    match v {
      _ if v.is_boolean() => Ok(Value::Bool(v.as_bool().unwrap())),
      _ if v.is_i64() => Ok(Value::Int(v.as_i64().unwrap())),
      _ if v.is_u64() => Ok(Value::UInt(v.as_u64().unwrap())),
      _ if v.is_f64() => Ok(Value::Float(v.as_f64().unwrap())),
      _ if v.is_string() => Ok(Value::Str(v.as_str().unwrap().to_owned())),
      _ => Err(anyhow!("failed to convert JSON to primitive value")),
    }
  }
}

/// Encoded width is a constant property of a compressor. It defines the size of
/// the compressed values produced by the compressor in number of bits. It is
/// used by the encoding system to determine whether to encapsulate the encoded
/// data in a fixed or variable width block. See [`Block`] for more information
/// on how data is stored inside a [compressed object].
///
/// [`Block`]: crate::data::Block
/// [compressed object]: crate::data::CompressedObject
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EncodedWidth {
  Fixed(usize),
  Variable,
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

  /// How many bits an encoded value produced by this compressor will take up.
  ///
  /// A compressor's encoded width **must** be deterministic as it is used once
  /// to first encode data and then second time (in a different invocation of
  /// the program) to decode the data.
  fn encoded_width(&self) -> EncodedWidth;
}

/// Returns an error stating that a given value type cannot be handled by the
/// compressor.
fn unexpected_type(value: Value, hint: &str) -> Error {
  anyhow!(
    "unexpected value type: {}, expected {}",
    value.typename(),
    hint
  )
}

use crate::comp::*;

/// A compressor for boolean types.
///
/// Compresses a boolean's string representation into a single bit.
pub struct BooleanCompressor;

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

  fn encoded_width(&self) -> EncodedWidth {
    EncodedWidth::Fixed(1)
  }
}

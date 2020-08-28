use crate::comp::*;

// FIXME: this compressor only works on strings, should probably rename it to
//  something else as I don't plan on letting it support other value types

/// The identity compressor doesn't perform any compression and instead passes
/// along any input data unmodified. It only accepts string values.
pub struct IdentityCompressor;

impl Compressor for IdentityCompressor {
  fn compress(&self, value: Value) -> Result<BitVec> {
    match value {
      Value::Str(s) => {
        let b = BitVec::from_bytes(s.as_bytes());
        Ok(b)
      },
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

  fn encoded_width(&self) -> EncodedWidth {
    EncodedWidth::Variable
  }
}

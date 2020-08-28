use crate::comp::*;

/// Compressor for enumerations of string variants.
///
/// Takes a fixed set of variants and compresses them into unique integer values
/// represented using the minimum required number of bits.
pub struct EnumCompressor {
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

  fn encoded_width(&self) -> EncodedWidth {
    let width = math::required_bit_width(self.variants.len());
    EncodedWidth::Fixed(width)
  }
}

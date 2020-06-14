//! Encoder for Universally Unique Identifiers (UUID).

use super::{Compressor, Glob, Result};
use uuid::Uuid;

/// An encoder specialized in handling Universally Unique Identifiers (UUID).
///
/// The encoded format is the raw binary which makes up the UUID (i.e., a
/// `u128`). Any parsable UUID format is acceptable as input though the output
/// after decompressing is a UUID in hyphenated form.
pub struct UuidCompressor;

impl Compressor for UuidCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    let str_input = String::from_utf8_lossy(input);
    let uuid = Uuid::parse_str(str_input.as_ref())?;
    let uuid_width = 128; // UUIDs are always 128 bits long.
    Ok(Glob::new(uuid_width, uuid.as_bytes().to_vec()))
  }

  fn decompress(&self, glob: Glob) -> Result<Vec<u8>> {
    let uuid = Uuid::from_slice(glob.data.as_slice())?;
    let uuid_str = uuid.to_hyphenated().to_string();
    Ok(uuid_str.into_bytes())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn compress_and_decompress_uuid() {
    let uuid_str = "0a53309c-98d7-43cb-98e8-89562adf0f0c";
    let result = UuidCompressor.compress(uuid_str.as_bytes());
    assert!(result.is_ok());

    let glob = result.unwrap();
    let result = UuidCompressor.decompress(glob);
    assert!(result.is_ok());

    let bytes = result.unwrap();
    let decompressed_str = String::from_utf8_lossy(&bytes);
    assert_eq!(uuid_str, decompressed_str);
  }
}

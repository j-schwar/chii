//! String compressors using Huffman encoding.

use super::{Compressor, Glob, Result};
use bit_vec::BitVec;
use huffman_compress::{Book, CodeBuilder, Tree};
use std::iter::FromIterator;

pub struct HuffmanCompressor {
  book: Book<u8>,
  tree: Tree<u8>,
}

impl HuffmanCompressor {
  pub fn from_weights<I: Iterator<Item = (u8, usize)>>(weights: I) -> Self {
    let (book, tree) = CodeBuilder::from_iter(weights).finish();
    HuffmanCompressor { book, tree }
  }

  pub fn ascii() -> Self {
    Self::from_weights(ASCII_WEIGHTS.iter().cloned())
  }
}

impl Compressor for HuffmanCompressor {
  fn compress(&self, input: &[u8]) -> Result<Glob> {
    let mut buffer = BitVec::new();
    for byte in input {
      self.book.encode(&mut buffer, byte)?;
    }
    Ok(Glob::new(buffer.len(), buffer.to_bytes()))
  }

  fn decompress(&self, glob: Glob) -> Result<Vec<u8>> {
    let mut buffer = BitVec::from_bytes(&glob.data);
    // Trim off excess bits
    while buffer.len() != glob.width {
      buffer.pop();
    }
    let decoded = self.tree.unbounded_decoder(&buffer).collect();
    Ok(decoded)
  }
}

const ASCII_WEIGHTS: [(u8, usize); 128] = [
  (0, 1),
  (1, 1),
  (2, 1),
  (3, 1),
  (4, 1),
  (5, 1),
  (6, 1),
  (7, 1),
  (8, 1),
  (b'\t', 1),
  (b'\n', 124457),
  (11, 1),
  (12, 1),
  (b'\r', 1),
  (14, 1),
  (15, 1),
  (16, 1),
  (17, 1),
  (18, 1),
  (19, 1),
  (20, 1),
  (21, 1),
  (22, 1),
  (23, 1),
  (24, 1),
  (25, 1),
  (26, 1),
  (27, 1),
  (28, 1),
  (29, 1),
  (30, 1),
  (31, 1),
  (b' ', 1293935),
  (b'!', 8845),
  (b'"', 471),
  (b'#', 2),
  (b'$', 1),
  (b'%', 2),
  (b'&', 22),
  (b'\'', 31070),
  (b'(', 629),
  (b')', 630),
  (b'*', 64),
  (b'+', 1),
  (b',', 83175),
  (b'-', 8075),
  (b'.', 78026),
  (b'/', 6),
  (b'0', 300),
  (b'1', 929),
  (b'2', 367),
  (b'3', 331),
  (b'4', 94),
  (b'5', 83),
  (b'6', 64),
  (b'7', 42),
  (b'8', 41),
  (b'9', 949),
  (b':', 1828),
  (b';', 17200),
  (b'<', 469),
  (b'=', 2),
  (b'>', 442),
  (b'?', 10477),
  (b'@', 9),
  (b'A', 44487),
  (b'B', 15414),
  (b'C', 21498),
  (b'D', 15684),
  (b'E', 42584),
  (b'F', 11714),
  (b'G', 11165),
  (b'H', 18463),
  (b'I', 55807),
  (b'J', 2068),
  (b'K', 6197),
  (b'L', 23859),
  (b'M', 15873),
  (b'N', 27339),
  (b'O', 33210),
  (b'P', 11940),
  (b'Q', 1179),
  (b'R', 28971),
  (b'S', 34012),
  (b'T', 39801),
  (b'U', 14130),
  (b'V', 3581),
  (b'W', 16497),
  (b'X', 607),
  (b'Y', 9100),
  (b'Z', 533),
  (b'[', 2086),
  (b'\\', 1),
  (b']', 2078),
  (b'^', 1),
  (b'_', 72),
  (b'`', 2),
  (b'a', 244665),
  (b'b', 46544),
  (b'c', 66689),
  (b'd', 133780),
  (b'e', 404622),
  (b'f', 68804),
  (b'g', 57036),
  (b'h', 218407),
  (b'i', 198185),
  (b'j', 2713),
  (b'k', 29213),
  (b'l', 146162),
  (b'm', 95581),
  (b'n', 215925),
  (b'o', 281392),
  (b'p', 46526),
  (b'q', 2405),
  (b'r', 208895),
  (b's', 214979),
  (b't', 289976),
  (b'u', 114819),
  (b'v', 33990),
  (b'w', 72895),
  (b'x', 4689),
  (b'y', 85272),
  (b'z', 1100),
  (b'{', 1),
  (b'|', 34),
  (b'}', 3),
  (b'~', 2),
  (127, 1),
];

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn sanity_test_simple_string() -> Result<()> {
    let compressor = HuffmanCompressor::from_weights(vec![
      (b'a', 100),
      (b'b', 50),
      (b'c', 25),
      (b'd', 10),
    ]);

    let input = b"aaaabbcdcbaaacaabb";
    let glob = compressor.compress(input)?;
    let result = compressor.decompress(glob)?;
    assert_eq!(&result, input);
    Ok(())
  }
}

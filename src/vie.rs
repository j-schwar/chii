//! Variable-Width Integer Encoding (VIE): a UTF-8 style encoding for integer
//! values which allows for (theoretically) unbounded integers to be encoded
//! in an efficient way optimizing for smaller values.
//!
//! > While this encoding theoretically supports unbounded integers, this
//! > implementation only supports up to 64-bit integer values for simplicity.

use crate::int::{FixedWidthInteger, LittleEndian};
use crate::math;
use num_traits::PrimInt;

/// A code point in the variable-width integer encoding encodes an integer
/// value as a string of bytes; not too dissimilar from little endian
/// encoding with least significant bytes appearing first. However, unlike
/// little endian encoding the number of bytes used to encode the number is
/// relative to it's value and not statically determined by the type of
/// integer being encoded.
///
/// The encoding is similar to UTF-8 with the highest bit of the byte reserved
/// as a prefix; with 1 denoting the next byte is a continuation of this code
/// point and 0 meaning that this byte is the end of the code point. The
/// remaining 7 bits of each byte hold the actual value of the integer in a
/// little endian fashion.
///
/// # Examples
///
/// For a simple example lets look at encoding the number 131 which is
/// `1000 0011` in binary. Usually this value fits within a single byte, but
/// with this encoding we have to spread it over 2 bytes as shown bellow:
///
/// ```text
/// 1000 0011   0000 0001
/// ^~~~ ~~~~           #
/// ```
///
/// In the above example, the highest bit of the first byte (marked with `^`)
/// is the prefix bit which says that the next byte is apart of this code
/// point. The 7 bits after it (marked with `~`) are the lower 7 bits of our
/// value (131) which are left intact. The highest bit of 131 has been chopped
/// off the original byte and placed in the lowest position of the second byte.
/// The highest bit of the second byte is 0 which means that this is the last
/// byte of this code point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodePoint {
  bytes: Vec<u8>,
}

impl CodePoint {
  /// The number of bytes taken up by this code point.
  #[inline]
  pub fn count(&self) -> usize {
    self.bytes.len()
  }

  /// Reference to the bytes which make up this code point.
  #[inline]
  pub fn bytes(&self) -> &[u8] {
    &self.bytes[..]
  }

  /// Decodes this code point into a native integer type.
  ///
  /// Returns `None` if the value of this code point is too large to store in
  /// the requested integer. For example, trying to decode a code point with
  /// value 300 in a `u8`.
  pub fn decode<I>(&self) -> Option<I>
  where
    I: FixedWidthInteger + LittleEndian,
  {
    // Strip prefix bits from code point bytes.
    let u7_vec = self.bytes.iter().map(|x| x & 0x7f).collect::<Vec<u8>>();

    // Convert u7 slice to u8 little endian vector.
    let mut le_bytes = u7_to_u8(u7_vec);

    // Trim trailing zero bytes from the little endian representation up until
    // the byte width of the integer we are trying to create.
    let byte_width = I::WIDTH / 8;
    while le_bytes.last() == Some(&0) && le_bytes.len() > byte_width {
      le_bytes.pop();
    }

    // If the little endian vector is too large for the required integer size
    // return None.
    if le_bytes.len() > byte_width {
      return None;
    }

    // If needed, pad the little endian vector with zero bytes until it is the
    // required size to convert to a native type.
    while le_bytes.len() != byte_width {
      le_bytes.push(0);
    }

    // Construct native type from little endian vector.
    I::from_le_bytes(le_bytes.as_slice())
  }
}

impl<I> From<I> for CodePoint
where
  I: LittleEndian + PrimInt,
{
  /// Constructs a code point from an integer value.
  fn from(x: I) -> Self {
    // Special case for 0 values.
    if x == I::zero() {
      return CodePoint { bytes: vec![0] };
    }

    let le_bytes = x.le_bytes();
    let mut u7_vec = u8_to_u7(le_bytes.as_slice());
    // Trim trailing zero bytes from the little endian `u7` vector.
    while u7_vec.last() == Some(&0) {
      u7_vec.pop();
    }

    debug_assert!(!u7_vec.is_empty());

    // To convert a u7 sequence to a code point we need to OR a 1 bit into the
    // free high bit of each byte except for the last one.
    for i in 0..u7_vec.len() - 1 {
      u7_vec[i] |= 0x80;
    }

    CodePoint { bytes: u7_vec }
  }
}

/// Converts a slice of bytes into a slice of u7 (unsigned 7-bit integers) by
/// continually masking off the high bit from each byte and shifting it into
/// the adjacent byte cascading the result of the shift down the slice.
fn u8_to_u7(bytes: &[u8]) -> Vec<u8> {
  // TODO: There is probably a more efficient algorithm to do this.
  debug_assert!(!bytes.is_empty());
  let mut vec: Vec<u8> = bytes.to_vec();
  let mut i = 0;
  while i != vec.len() {
    // Split off the high bit of the `i`th byte.
    let (value, mut carry_in) = split_high_bit(vec[i]);
    // Place back the new value into the vector.
    vec[i] = value;
    // Cascade, shift the carry of the previous shift into the next byte.
    for byte in vec.iter_mut().skip(i + 1) {
      // Shift this byte to the left by 1 to make room for the carry in.
      let (shifted, carry_out) = math::shl_with_carry(*byte, 1);
      // Combine the shifted result with the carry in giving us the new byte
      // for this position.
      let shifted = shifted | carry_in;
      // Place the new shifted value back into the vector.
      *byte = shifted;
      // If this shift overflowed, then we carry a 1 to the next byte,
      // otherwise we carry a zero.
      carry_in = carry_out;
    }

    // We've now chopped off the high bit of the byte in the `i`th position
    // and shifted it into the next byte, cascading the shift throughout the
    // rest of the bytes in the sequence.

    // Carry in now holds the carry out of the last shift, if it is one then
    // we need add a new byte to the result to hold it.
    if carry_in == 1 {
      vec.push(carry_in);
    }

    // Now we chop of the highest bit of the next byte, shifting it into the
    // next byte and so on...
    i += 1;
  }

  vec
}

/// Masks off the high bit of `x` returning it as the lowest bit in the second
/// tuple element.
fn split_high_bit(x: u8) -> (u8, u8) {
  (x & 0x7f, x >> 7)
}

/// Converts a vector of `u7` integers into a vector of `u8` integers.
fn u7_to_u8(mut u7_vec: Vec<u8>) -> Vec<u8> {
  debug_assert!(!u7_vec.is_empty());
  let mut vec = Vec::new();
  let mut i = 0;
  let mut borrow_amount = 1;
  loop {
    // Since we sometimes skip values (given certain circumstances), we check
    // to make sure we actually have data to work on this iteration.
    if i == u7_vec.len() {
      break;
    }

    // Get the value for this iteration.
    let value = u7_vec[i];
    // If there is no next value, add the current value to the result vector
    // and break, because we are done.
    if i + 1 == u7_vec.len() {
      vec.push(value);
      break;
    }

    // Borrow the required number of bits from the next value and OR them into
    // the top of the current one.
    let borrowed = if borrow_amount == 7 {
      // If we need to borrow the entire next value, reset the borrow amount
      // and add 1 to the index counter so we skip over the next value.
      borrow_amount = 1;
      i += 1;
      u7_vec[i] << 1
    } else {
      // Borrow the required amount of bits and shift them to the high part of
      // the byte so that we can OR them with the current value.
      let b = borrow_lower(u7_vec[i + 1], borrow_amount);
      let b = b << (8 - borrow_amount);
      // Shift the next value to the right so that it is ready for the next
      // loop iteration.
      u7_vec[i + 1] >>= borrow_amount;
      // Increment the borrow_amount because we will need to borrow 1 more bit
      // in the next iteration.
      borrow_amount += 1;
      // Return the shifted borrowed bits.
      b
    };

    // OR the borrowed bits into the current value and store it in the result
    // vector.
    let value = value | borrowed;
    vec.push(value);

    i += 1;
  }

  vec
}

/// Returns the lower `n` bits of `x`.
fn borrow_lower(x: u8, n: u8) -> u8 {
  debug_assert!(n <= 8);
  x & (0xff >> (8 - n))
}

#[cfg(test)]
mod test {
  use super::*;
  use proptest::prelude::*;

  #[test]
  fn code_point_from_u8_no_high_bit() {
    let cp = CodePoint::from(0x7fu8);
    assert_eq!(&[0x7f], cp.bytes());
  }

  #[test]
  fn code_point_from_u8_with_high_bit() {
    let cp = CodePoint::from(0xd9u8);
    assert_eq!(&[0xd9, 0x01], cp.bytes());
  }

  #[test]
  fn code_point_from_u16() {
    let cp = CodePoint::from(0x7081u16);
    assert_eq!(&[0x81, 0xe1, 0x01], cp.bytes());
  }

  #[test]
  fn code_point_from_zero() {
    let cp = CodePoint::from(0u64);
    assert_eq!(&[0u8], cp.bytes());
  }

  #[test]
  fn code_point_from_128() {
    let cp = CodePoint::from(128u16);
    assert_eq!(&[0x80, 0x01], cp.bytes());
  }

  #[test]
  fn code_point_encode_decode_0() {
    let cp = CodePoint::from(0u64);
    assert_eq!(Some(0), cp.decode::<u64>());
  }

  #[test]
  fn code_point_encode_decode_1() {
    let cp = CodePoint::from(1u32);
    assert_eq!(Some(1), cp.decode::<u32>());
  }

  #[test]
  fn code_point_encode_decode_128() {
    let cp = CodePoint::from(128u8);
    assert_eq!(Some(128), cp.decode::<u8>());
  }

  #[test]
  fn code_point_encode_decode_32768() {
    let cp = CodePoint::from(32768u16);
    assert_eq!(Some(32768), cp.decode::<u16>());
  }

  #[test]
  fn code_point_encode_decode_0x2_0000_0000_0000() {
    let value = 0x2_0000_0000_0000u64;
    let cp = CodePoint::from(value);
    assert_eq!(Some(value), cp.decode::<u64>());
  }

  #[test]
  fn code_point_count_for_u64_max_is_9() {
    let value = i64::MAX;
    let cp = CodePoint::from(value);
    assert_eq!(9, cp.count());
  }

  #[test]
  fn u8_to_u7_single_byte_no_high_bit() {
    let bytes = [0x7f];
    assert_eq!(&[0x7f], &u8_to_u7(&bytes)[..]);
  }

  #[test]
  fn u8_to_u7_single_byte_with_high_bit() {
    let bytes = [0xd9];
    assert_eq!(&[0x59, 0x01], &u8_to_u7(&bytes)[..])
  }

  #[test]
  fn u8_to_u7_two_bytes_with_no_final_carry_out() {
    let bytes = [0x01, 0x3f];
    assert_eq!(&[0x01, 0x7e], &u8_to_u7(&bytes)[..])
  }

  #[test]
  fn u8_to_u7_two_bytes_with_final_carry_out() {
    let bytes = [0x81, 0x70];
    assert_eq!(&[0x01, 0x61, 0x01], &u8_to_u7(&bytes)[..])
  }

  #[test]
  fn split_high_bit_with_no_high_bit() {
    assert_eq!((0x7f, 0x00), split_high_bit(0x7f));
  }

  #[test]
  fn split_high_bit_with_high_bit() {
    assert_eq!((0x5f, 0x01), split_high_bit(0xdf));
  }

  proptest! {
    #[test]
    fn prop_code_point_encode_decode_u8(x: u8) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<u8>());
    }

    #[test]
    fn prop_code_point_encode_decode_u16(x: u16) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<u16>());
    }

    #[test]
    fn prop_code_point_encode_decode_u32(x: u32) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<u32>());
    }

    #[test]
    fn prop_code_point_encode_decode_u64(x: u64) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<u64>());
    }

    #[test]
    fn prop_code_point_encode_decode_i8(x: i8) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<i8>());
    }

    #[test]
    fn prop_code_point_encode_decode_i16(x: i16) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<i16>());
    }

    #[test]
    fn prop_code_point_encode_decode_i32(x: i32) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<i32>());
    }

    #[test]
    fn prop_code_point_encode_decode_i64(x: i64) {
      let cp = CodePoint::from(x);
      assert_eq!(Some(x), cp.decode::<i64>());
    }

    #[test]
    fn prop_code_point_bytes_should_never_end_in_a_zero(x: u64) {
      let cp = CodePoint::from(x);
      assert!(cp.bytes().last() != Some(&0));
    }

    #[test]
    fn prop_code_point_last_byte_should_never_have_high_bit_set(x: u64) {
      let cp = CodePoint::from(x);
      let last = cp.bytes().last().unwrap();
      assert!(last & 0x80 == 0);
    }
  }
}

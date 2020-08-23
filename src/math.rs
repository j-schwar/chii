//! Math utilities.

use crate::int::FixedWidthInteger;
use num_traits::{PrimInt, Unsigned};

/// Unsigned integer division rounding away from zero.
pub fn div_ceil<I: PrimInt + Unsigned>(lhs: I, rhs: I) -> I {
  let x = lhs / rhs;
  if lhs % rhs != I::zero() {
    x + I::one()
  } else {
    x
  }
}

/// Shifts `lhs` to the left by `rhs` bits returning the result of the shift
/// along with the bits that were shifted out, which are shifted from the
/// high part of the byte to the low part.
pub fn shl_with_carry(lhs: u8, rhs: u8) -> (u8, u8) {
  debug_assert!(rhs <= 8);
  if rhs == 8 {
    (0, lhs)
  } else if rhs == 0 {
    (lhs, 0)
  } else {
    (lhs << rhs, lhs >> (8 - rhs))
  }
}

/// Shifts `v` to the left `n` bits.
///
/// If one thinks of `v` as the little-endian representation of an arbitrary
/// precision integer, then this operation is equivalent to a left shift by
/// `n` on that arbitrary precision integer.
///
/// Even if the carry out of the last byte is 0, an additional 0 value will be
/// appended to the resultant vector. This is to aid this function's primary
/// caller and may be a bit counter intuitive.
///
/// # Example
///
/// ```
/// # use chii::math::vec_shl;
/// let bytes = vec![0x73, 0x01];
/// let shifted = vec_shl(bytes, 2);
/// assert_eq!(vec![0xcc, 0x05, 0x00], shifted);
/// ```
pub fn vec_shl(mut v: Vec<u8>, n: u8) -> Vec<u8> {
  let mut carry_in = 0;
  for b in v.iter_mut() {
    let (shifted, carry_out) = shl_with_carry(*b, n);
    *b = shifted | carry_in;
    carry_in = carry_out;
  }
  if n % 8 != 0 {
    v.push(carry_in);
  }
  v
}

/// Returns the required number of bits needed to store `n` unique bit patterns.
///
/// For example, for 3 unique values two bits are needed: 00, 01, 10. For 4
/// values you still only need 2 bits (00, 01, 10, 11), but for 5 values you
/// would need 3 bits.
pub fn required_bit_width(n: usize) -> usize {
  n.next_power_of_two().trailing_zeros() as usize
}

/// Creates a bit mask with the lowest `n` bits set to 1 and the rest 0.
pub fn low_mask<I: PrimInt + FixedWidthInteger>(n: usize) -> I {
  !I::zero() >> (I::WIDTH - n)
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn div_ceil_1_2() {
    assert_eq!(1u32, div_ceil(1, 2));
  }

  #[test]
  fn div_ceil_2_2() {
    assert_eq!(1u32, div_ceil(2, 2));
  }

  #[test]
  fn div_ceil_3_2() {
    assert_eq!(2u32, div_ceil(3, 2));
  }

  #[test]
  fn shl_with_carry_0xd0() {
    assert_eq!((0x80, 0x06), shl_with_carry(0xd0, 3));
  }

  #[test]
  fn required_bit_width_6() {
    assert_eq!(3, required_bit_width(6));
  }

  #[test]
  fn required_bit_width_8() {
    assert_eq!(3, required_bit_width(8));
  }

  #[test]
  fn required_bit_width_97() {
    assert_eq!(7, required_bit_width(97))
  }

  #[test]
  fn vec_shl_out_zero() {
    let bytes = vec![0x80, 0x01];
    let shifted = vec_shl(bytes, 1);
    assert_eq!(vec![0, 3, 0], shifted);
  }

  #[test]
  fn vec_shl_with_extend_vec() {
    let bytes = vec![0x80];
    let shifted = vec_shl(bytes, 1);
    assert_eq!(vec![0, 1], shifted);
  }

  #[test]
  fn low_mask_3() {
    assert_eq!(0b0000_0111, low_mask::<u8>(3));
  }
}

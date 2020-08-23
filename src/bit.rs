//! Utility functions for dealing with bit vectors.

use crate::int::BigEndian;
pub use bit_vec::BitVec;

/// Extensions to `BitVec`.
pub trait BitVecExt {
  /// Constructs a `BitVec` from the bit-reversed big endian representation of
  /// an integer. This results in the least significant bit being at index 0 in
  /// the resultant `BitVec`.
  ///
  /// # Example
  ///
  /// ```
  /// # use chii::bit::{BitVec, BitVecExt};
  /// let b = BitVec::from_rev_be(0x83u16);
  /// assert_eq!(b.to_bytes(), &[0b1100_0001, 0b0000_0000]);
  /// ```
  fn from_rev_be<I>(i: I) -> Self
  where
    I: BigEndian;

  /// Converts a `BitVec` into an integer by interpreting the bits in the vector
  /// as the bit-reversed big endian representation of some integer type.
  ///
  /// This is the inverse operation of [`from_rev_be`].
  ///
  /// [`from_rev_be`]: BitVecExt::from_rev_be
  fn to_rev_be<I>(&self) -> Option<I>
  where
    I: BigEndian;

  /// Zero extends or truncates this `BitVec` to the desired length.
  fn zext_or_trunc(&mut self, len: usize);
}

impl BitVecExt for BitVec {
  fn from_rev_be<I>(i: I) -> Self
  where
    I: BigEndian,
  {
    let rev = i.reverse_bits();
    Self::from_bytes(&rev.be_bytes())
  }

  fn to_rev_be<I>(&self) -> Option<I>
  where
    I: BigEndian,
  {
    let bytes = self.to_bytes();
    I::from_be_bytes(&bytes).map(I::reverse_bits)
  }

  fn zext_or_trunc(&mut self, len: usize) {
    if self.len() < len {
      self.grow(len - self.len(), false);
    } else {
      self.truncate(len);
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use proptest::prelude::*;

  #[test]
  fn from_rev_be() {
    let b = BitVec::from_rev_be(0x83u16);
    assert_eq!(b.to_bytes(), &[0b1100_0001, 0b0000_0000]);
  }

  #[test]
  fn zext_or_trunc_zero_extend() {
    let mut b = BitVec::from_rev_be(0x83u8);
    b.zext_or_trunc(10);
    assert_eq!(b.len(), 10);
    assert_eq!(b.to_bytes(), &[0b1100_0001, 0b0000_0000]);
  }

  #[test]
  fn zext_or_trunc_truncate() {
    let mut b = BitVec::from_rev_be(0x83u8);
    b.zext_or_trunc(7);
    assert_eq!(b.len(), 7);
    assert_eq!(b.to_bytes(), &[0b1100_0000]);
  }

  proptest! {
    #[test]
    fn prop_to_rev_be_inverse_of_from_rev_be(x: u16) {
      let b = BitVec::from_rev_be(x);
      let y = b.to_rev_be::<u16>();
      assert_eq!(Some(x), y);
    }
  }
}

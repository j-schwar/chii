//! N-bit integer implementation and various integer related traits.
//!
//! Note that this implementation only supports N <= 64 for simplicity.

use num_traits::PrimInt;
use smallvec::SmallVec;
use std::convert::TryInto;

/// Trait for integer types which expose a little endian byte representation.
pub trait LittleEndian: Sized {
  fn from_le_bytes(bytes: &[u8]) -> Option<Self>;

  /// The little endian byte representation of this integer.
  fn le_bytes(&self) -> SmallVec<[u8; 8]>;
}

macro_rules! impl_little_endian {
  ( $($t:ty),* ) => {
    $(
      impl LittleEndian for $t {
        fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
          let arr = bytes.try_into().ok()?;
          Some(<$t>::from_le_bytes(arr))
        }

        fn le_bytes(&self) -> SmallVec<[u8; 8]> {
          SmallVec::from_slice(&self.to_le_bytes())
        }
      }
    )*
  };
}

impl_little_endian!(u8, u16, u32, u64, i8, i16, i32, i64);

/// Trait for integers with a fixed width.
pub trait FixedWidthInteger {
  /// The width of this integer type in bits.
  const WIDTH: usize;
}

macro_rules! impl_fixed_width_integer {
  ( $($t:ty => $v:tt),* ) => {
    $(
      impl FixedWidthInteger for $t {
        const WIDTH: usize = $v;
      }
    )*
  };
}

impl_fixed_width_integer! {
  u8 => 8,
  i8 => 8,
  u16 => 16,
  i16 => 16,
  u32 => 32,
  i32 => 32,
  u64 => 64,
  i64 => 64
}

/// Appends 0 elements to the end of `vec` until it's length is equal to `n`.
///
/// If `vec` already contains more elements then `n` then this function does
/// nothing.
pub fn pad_with_zero<I: PrimInt>(n: usize, vec: &mut Vec<I>) {
  while vec.len() < n {
    vec.push(I::zero());
  }
}

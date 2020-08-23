//! Various integer related traits.

use std::convert::TryInto;

/// Trait for integers with a fixed width.
pub trait FixedWidthInteger {
  /// The width of this integer type in bits.
  const WIDTH: usize;

  /// Returns a version of this integer with all bits reversed.
  fn reverse_bits(self) -> Self;
}

macro_rules! impl_fixed_width_integer {
  ( $($t:ty => $v:tt),* ) => {
    $(
      impl FixedWidthInteger for $t {
        const WIDTH: usize = $v;

        fn reverse_bits(self) -> Self {
          self.reverse_bits()
        }
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

/// Trait for integer types which expose a big endian byte representation.
pub trait BigEndian: Sized + FixedWidthInteger {
  /// Constructs this integer type from its big endian byte representation.
  ///
  /// Returns `None` if `bytes` doesn't contain the exact number of bytes
  /// required for the implementing integer type (e.g., 8 bytes for u64).
  fn from_be_bytes(bytes: &[u8]) -> Option<Self>;

  /// The big endian byte representation of this integer.
  fn be_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_big_endian {
  ( $($t:ty),* ) => {
    $(
      impl BigEndian for $t {
        fn from_be_bytes(bytes: &[u8]) -> Option<Self> {
          let arr = bytes.try_into().ok()?;
          Some(<$t>::from_be_bytes(arr))
        }

        fn be_bytes(&self) -> Vec<u8> {
          self.to_be_bytes().to_vec()
        }
      }
    )*
  }
}

impl_big_endian!(u8, u16, u32, u64, i8, i16, i32, i64);

/// Trait for integer types which expose a little endian byte representation.
pub trait LittleEndian: Sized + FixedWidthInteger {
  fn from_le_bytes(bytes: &[u8]) -> Option<Self>;

  /// The little endian byte representation of this integer.
  fn le_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_little_endian {
  ( $($t:ty),* ) => {
    $(
      impl LittleEndian for $t {
        fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
          let arr = bytes.try_into().ok()?;
          Some(<$t>::from_le_bytes(arr))
        }

        fn le_bytes(&self) -> Vec<u8> {
          self.to_le_bytes().to_vec()
        }
      }
    )*
  };
}

impl_little_endian!(u8, u16, u32, u64, i8, i16, i32, i64);

use crate::core::math;

/// A glob of data in some arbitrary format which holds the actual data of the
/// compressed object either as part of a field or a list.
///
/// The compressed object format is constructed in such a way that we do not
/// need to care what format the glob data is in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Glob {
  /// The width of this glob in number of bits.
  pub width: usize,
  /// The data that makes up this glob.
  ///
  /// We don't care what format it is in.
  pub data: Vec<u8>,
}

impl Glob {
  /// Constructs a new instance.
  ///
  /// Trims `data` so that only the required number of bytes are stored in the
  /// glob. While this is not entirely necessary, it makes testing more
  /// intuitive especially when we just throw the little endian representation
  /// into a glob without any thought.
  ///
  /// # Panics
  ///
  /// Panics if `width` is zero or if `data` does not contain enough bytes to
  /// hold the number of bits that `width` says are in this glob.
  pub fn new(width: usize, data: Vec<u8>) -> Self {
    if width == 0 {
      panic!("zero width globs are not allowed");
    }

    if data.len() < width / 8 {
      panic!("not enough data to hold {} bits", width);
    }

    let mut glob = Glob { width, data };
    glob.truncate_data();
    glob
  }

  /// Appends the bits of some other glob into this glob.
  ///
  /// The result is a packed sequence of bits with a length equal to the sum of
  /// the two globs. No padding introduced between the two globs, they are
  /// packed tightly together.
  ///
  /// # Example
  ///
  /// ```
  /// # use dsc::glob::Glob;
  /// // Start with a 3-bit wide glob
  /// //   001
  /// let mut glob = Glob::new(3, vec![0x01]);
  ///
  /// // Append a 4-bit wide glob
  /// //  1001
  /// glob.append(Glob::new(4, vec![0x09]));
  ///
  /// // Get a 7-bit wide glob
  /// //  1001_001
  /// assert_eq!(Glob::new(7, vec![0x49]),  glob);
  /// ```
  pub fn append(&mut self, mut other: Glob) {
    // Appending is simple if this glob's width is a whole number of bytes.
    if self.width % 8 == 0 {
      self.data.append(&mut other.data);
      self.width += other.width;
      return;
    }

    // Shift other's data to the left by the number of valid bits in this glob's
    // last byte to make room so that we can OR them together.
    let shift_amount = self.width % 8;
    let shifted = math::vec_shl(other.data, shift_amount as u8);
    debug_assert!(!shifted.is_empty());

    // OR the first byte of `other` into `self`'s last byte.
    let last = self.data.last_mut().unwrap();
    *last |= shifted[0];

    // Append the remaining bytes from other.
    for byte in &shifted[1..] {
      self.data.push(*byte);
    }

    // The width of the new glob is equal to the sum of the two widths.
    self.width += other.width;

    // We may have some excess bytes on the end of the data vector so we need to
    // truncate it to the appropriate width.
    self.truncate_data();
  }

  /// Truncates `self`'s data vector to the the minimum number of bytes required
  /// to hold its declared width.
  fn truncate_data(&mut self) {
    let required_bytes = math::div_ceil(self.width, 8);
    while self.data.len() > required_bytes {
      self.data.pop();
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn byte_aligned_glob_append() {
    let mut a = Glob::new(8, vec![1]);
    let b = Glob::new(8, vec![2]);
    a.append(b);
    assert_eq!(Glob::new(16, vec![1, 2]), a);
  }

  #[test]
  fn appending_bit_globs() {
    let mut glob = Glob::new(1, vec![0]);
    glob.append(Glob::new(1, vec![0]));
    glob.append(Glob::new(1, vec![0]));
    glob.append(Glob::new(1, vec![1]));
    glob.append(Glob::new(1, vec![1]));
    glob.append(Glob::new(1, vec![0]));
    assert_eq!(Glob::new(6, vec![0b011000]), glob);
  }

  #[test]
  fn appending_variable_width_globs() {
    let mut glob = Glob::new(3, vec![1]);
    glob.append(Glob::new(4, vec![0]));
    glob.append(Glob::new(2, vec![3]));
    assert_eq!(Glob::new(9, vec![0b1_0000_001, 0b1]), glob);
  }

  #[test]
  fn appending_8bit_glob_to_3bit_glob() {
    let mut glob = Glob::new(3, vec![0b100]);
    glob.append(Glob::new(8, vec![0x10]));
    assert_eq!(Glob::new(11, vec![0b10000_100, 0x000]), glob);
  }
}

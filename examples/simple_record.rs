//! This example showcases encoding a simple record object into binary.

use dsc::co::*;
use dsc::core::math;
use std::fmt::{Binary, Formatter, Result as FmtResult};

/// Vector wrapper to allow us to print a vector of bytes as binary.
struct V<B: Binary>(Vec<B>);

impl<B: Binary> Binary for V<B> {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    let vec = &self.0;
    for (i, x) in vec.iter().enumerate() {
      if i != 0 {
        write!(f, " ")?;
      }
      x.fmt(f)?;
    }
    Ok(())
  }
}

fn main() {
  // Create a new compressed record object to add data to.
  let mut co = CompressedObject::new_record();

  // Add some data to the record, here we are adding field #4 with 16 bits of
  // data. The field values 0 through 3 are reserved control markers so we
  // cannot use them as field numbers.
  co.push_data(Marker::Field(4), Glob::new(16, vec![0xff, 0xff]));

  // Globs don't need to be whole bytes.
  co.push_data(Marker::Field(5), Glob::new(7, vec![0x71]));

  // With this, we've constructed our record object with 2 fields now we need
  // to encode it.

  // The width of encoded markers is variable and will usually be determined by
  // the schema used to compress the object. Here however, we simply set it to
  // be the required bit width for the largest field number which in this case
  // is #5. The marker width here is 3 bits.
  let marker_width = math::required_bit_width(5);

  // The compressed object can be encoded using the `into_bytes` method.
  let bytes = co.into_bytes(marker_width);

  println!("{:08b}", V(bytes));
  // 00000001 00100001 11111110 11111111 01111011 00010000 00000111

  // Let's break apart this binary blob to see what it looks like. First off it
  // is important to note each byte is indexed from right to left and the bytes
  // themselves are indexed left to right; confusing yes I know. It looks
  // something like this:
  //
  //	 byte 0   byte 1   byte 2   byte 3   byte 4   byte 5   byte 6
  // 	00000001 00100001 11111110 11111111 01111011 00010000 00000111
  //         ^       ^
  //     bit 0   bit 1 and so on...
  //
  // The individual components ordered as they appear in the binary is like so:
  //
  // 	001			 - record marker
  //	000	 		 - null marker
  // 	100 		 - marker field (#4)
  //	00010000 - vie length (16)
  // 	11111111 - glob byte 0
  // 	11111111 - glob byte 1
  //  101 		 - marker field (#5)
  // 	00000111 - vie length (7)
  //  1110001  - glob byte 0
  // 	00000 	 - padding
  //
  // All these components are packed together with no spacing in between.
}

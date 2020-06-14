//! This module defines the data representation of  Compressed Object
//! (CO) files.

use crate::core::math;
use crate::core::CodePoint;

#[cfg(test)]
mod test;

/// Markers are predetermined values which mark the starts of the various
/// sections in a compress object file: records, lists, fields, elements.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Marker {
  /// Null marker; a reserved marker encoded as the cardinal value 0.
  Null,

  /// Marker denoting the start of a list; encoded as the cardinal value 1.
  ///
  /// `Record` markers are immediately followed by a``Field` marker which
  /// denotes the field this record belongs to, or a `Null` marker if this is
  /// an anonymous record.
  Record,

  /// Marker denoting the start of a list; encoded as the cardinal value 2.
  ///
  /// Like with `Record`, `List` markers are immediately followed by a `Field`
  /// marker denoting the field this list belongs to or a `Null` marker for
  /// anonymous lists.
  List,

  /// Marker denoting a list element; encoded as the cardinal value 3.
  Element,

  /// Marker denoting a record field.
  ///
  /// The possible set of fields is determined by the schema which assigns each
  /// field name a cardinal value different from the 4 reserved values used by
  /// the builtin markers.
  Field(u32),
}

impl Marker {
  /// Returns the encoded value for this marker.
  ///
  /// An encoder will later truncate this value to the minimum required size
  /// before creating a binary file. As such, we don't bother retaining the
  /// marker size determined by the schema within marker objects.
  ///
  /// # Panics
  ///
  /// Panics if `self` is a `Field` variant and contains a reserved value
  /// (i.e., 0, 1, 2, or 3).
  pub fn value(&self) -> u32 {
    use Marker::*;
    match self {
      Null => 0,
      Record => 1,
      List => 2,
      Element => 3,
      Field(v) => {
        if (0..4).contains(v) {
          panic!("field contains a reserved value: {}", v);
        }
        *v
      }
    }
  }

  /// Converts this marker into a binary glob.
  ///
  /// The width of the marker is determined by the number of possible fields
  /// defined in the external schema. As such, the glob width must be supplied
  /// before the marker can be converted.
  pub fn into_glob(self, width: usize) -> Glob {
    let bytes = self.value().to_le_bytes().to_vec();
    Glob::new(width, bytes)
  }

  /// True if `self` is a `Null` variant.
  pub fn is_null(&self) -> bool {
    match self {
      Marker::Null => true,
      _ => false,
    }
  }

  /// True if `self` is a `Record` variant.
  pub fn is_record(&self) -> bool {
    match self {
      Marker::Record => true,
      _ => false,
    }
  }

  /// True if `self` is a `List` variant.
  pub fn is_list(&self) -> bool {
    match self {
      Marker::List => true,
      _ => false,
    }
  }

  /// True if `self` is a `Element` variant.
  pub fn is_element(&self) -> bool {
    match self {
      Marker::Element => true,
      _ => false,
    }
  }

  /// True if `self` is a `Field` variant.
  pub fn is_field(&self) -> bool {
    match self {
      Marker::Field(..) => true,
      _ => false,
    }
  }
}

/// The `Length` section of a COF file prefixes a `Glob` and holds the length
/// of said glob in number of bits.
///
/// Instead of using a fixed width integer to store the length, we instead use
/// a [variable-width integer encoding](../core/vie/struct.CodePoint.html)
/// similar to UTF-8.
#[derive(Clone, Debug)]
pub struct Length(CodePoint);

impl Length {
  /// Converts this object into a binary glob.
  pub fn into_glob(self) -> Glob {
    let width = self.0.count() * 8;
    Glob::new(width, self.0.bytes().to_vec())
  }
}

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
  /// # use dsc::co::Glob;
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

/// Blocks represent larger structures within COF files. A COF file can be
/// thought of as simply a sequence of blocks packed tightly together.
#[derive(Clone, Debug)]
pub enum Block {
  /// Header blocks mark the start of a new record or list.
  ///
  /// The first marker determines whether this is a list or a record and the
  /// second marker is the containing field if the parent is a record or the
  /// `Element` marker if the parent element is a list.
  Header(Marker, Marker),

  /// Data blocks hold actual data for fields or list elements.
  ///
  /// The first marker is the field name if the parent is a record or the
  /// `Element` marker if it's a list. The length part holds the length of
  /// the following glob in number of bits. And, finally, the glob part holds
  /// the actual data.
  Data(Marker, Length, Glob),

  /// Terminator blocks mark the end of a nested record or list and are encoded
  /// as a single `Null` marker.
  Terminator,
}

impl Block {
  /// Converts this block into a binary glob.
  ///
  /// Since the width of `Marker` elements is determined by the external schema,
  /// it must be supplied in order to convert a block into a glob.
  pub fn into_glob(self, marker_width: usize) -> Glob {
    use Block::*;
    match self {
      Header(m1, m2) => {
        let mut glob = m1.into_glob(marker_width);
        glob.append(m2.into_glob(marker_width));
        glob
      }

      Data(m, l, g) => {
        let mut glob = m.into_glob(marker_width);
        glob.append(l.into_glob());
        glob.append(g);
        glob
      }

      Terminator => Marker::Null.into_glob(marker_width),
    }
  }
}

/// A compressed object is simply a sequence of blocks arranged is a particular
/// pattern.
///
/// Every compressed object must start with a `Header` block which defines the
/// root structure (either a record or list). Following that, an arbitrary
/// number of `Data` blocks can be added which describe the fields/elements of
/// the object.
///
/// Nested structures (i.e., records/lists within records/lists) must first
/// start with a new `Header` block which describes the type of the nested
/// object along with the field it belongs to. This new header can then be
/// followed by an arbitrary number of `Data` objects which describe the data
/// of this nested object. After all of the nested object's fields, a
/// `Terminator` block must be added. This signifies the end of this nested
/// object. Any `Data` blocks placed after this terminator will be treated as
/// part of the parent object.
///
/// The root object (i.e., the initial header) does not need to be terminated.
#[derive(Clone, Debug)]
pub struct CompressedObject {
  blocks: Vec<Block>,
}

impl CompressedObject {
  /// Constructs a new, empty object.
  ///
  /// When creating compressed objects by hand, use of `new_record` or
  /// `new_list` is encouraged over the use of this function.
  pub fn new() -> Self {
    CompressedObject { blocks: Vec::new() }
  }

  /// Constructs a new compressed object with a header marking it as a record
  /// type object.
  ///
  /// Additional blocks can be appended to this object to add fields.
  pub fn new_record() -> Self {
    let header = Block::Header(Marker::Record, Marker::Null);
    CompressedObject {
      blocks: vec![header],
    }
  }

  /// Constructs a new compressed object with a header marking it as a list
  /// type object.
  pub fn new_list() -> Self {
    let header = Block::Header(Marker::List, Marker::Null);
    CompressedObject {
      blocks: vec![header],
    }
  }

  /// Appends a new block to this object.
  ///
  /// The block is not validated to ensure that it is valid in the current
  /// context. To validate the integrity of a compressed object the `validate`
  /// method may be used.
  pub fn push(&mut self, block: Block) {
    self.blocks.push(block)
  }

  /// Begins a new nested record object associated with `field`.
  ///
  /// # Panics
  ///
  /// Panics if `field` is not a `Field` or `Element` variant.
  pub fn begin_nested_record(&mut self, field: Marker) {
    if !field.is_field() && !field.is_element() {
      panic!(
        "nested record requires a field or element marker, given: {:?}",
        field
      );
    }
    let header = Block::Header(Marker::Record, field);
    self.push(header);
  }

  /// Begins a new nested list object associated with `field`.
  ///
  /// # Panics
  ///
  /// Panics if `field` is not a `Field` or `Element` variant.
  pub fn begin_nested_list(&mut self, field: Marker) {
    if !field.is_field() && !field.is_element() {
      panic!(
        "nested list requires a field or element marker, given: {:?}",
        field
      );
    }
    let header = Block::Header(Marker::List, field);
    self.push(header);
  }

  /// Appends a `Terminator` block to end a nested object.
  pub fn end_nested_object(&mut self) {
    self.push(Block::Terminator);
  }

  /// Appends a data block associated with a given field.
  ///
  /// The `Length` section is inferred from the width of `glob`.
  ///
  /// # Panics
  ///
  /// Panics if `field` is not a `Field` or `Element` variant.
  pub fn push_data(&mut self, field: Marker, glob: Glob) {
    if !field.is_field() && !field.is_element() {
      panic!(
        "data blocks require a field or element marker, given: {:?}",
        field
      );
    }

    let length = Length(CodePoint::from(glob.width as u64));
    let block = Block::Data(field, length, glob);
    self.push(block);
  }

  /// Validates the integrity of this compressed object.
  pub fn validate(&self) -> Result<(), ValidationError> {
    Validator::run(self)
  }

  /// Converts this compressed object to a sequence of bytes which may, in turn,
  /// be writen to a binary file to persist the object.
  ///
  /// # Panics
  ///
  /// Panics if `self` is an invalid compressed object. This can be checked by
  /// using the `validate` method.
  pub fn into_bytes(self, marker_width: usize) -> Vec<u8> {
    if self.blocks.is_empty() {
      return Vec::new();
    }

    // Convert blocks into globs and pack them together.
    let mut iter = self.blocks.into_iter();
    let mut glob = iter.next().unwrap().into_glob(marker_width);
    for block in iter {
      glob.append(block.into_glob(marker_width));
    }

    // Return the glob's data, we don't care about it's width anymore.
    glob.data
  }
}

impl Default for CompressedObject {
  /// The default compressed object is an empty object.
  fn default() -> Self {
    CompressedObject::new()
  }
}

#[derive(Debug)]
pub enum ValidationError<'a> {
  UnexpectedBlock(&'a Block, usize, &'static str),
  UnexpectedMarker(&'a Marker, usize, &'static str),
  UnexpectedTerminator(usize),
  MalformedHeader(&'a Block, usize, &'static str),
  WrongMarkerType(&'a Marker, usize, &'static str),
  LengthMismatch(&'a Length, &'a Glob, usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
  Init,
  Root(Marker),
  Nested(Marker, Box<State>),
  End,
}

struct Validator<'a> {
  obj: &'a CompressedObject,
  index: usize,
  state: State,
}

impl<'a> Validator<'a> {
  pub fn run(obj: &CompressedObject) -> Result<(), ValidationError> {
    let mut validator = Validator::new(obj);
    loop {
      let new_state = validator.advance_state()?;
      if new_state == State::End {
        break;
      }
      validator.state = new_state;
    }
    Ok(())
  }

  fn new(obj: &'a CompressedObject) -> Self {
    Validator {
      obj,
      index: 0,
      state: State::Init,
    }
  }

  fn advance_state(&mut self) -> Result<State, ValidationError<'a>> {
    use State::*;
    use ValidationError::*;
    match self.state.clone() {
      Init => match self.consume_block() {
        Some(b @ &Block::Header(m, Marker::Null)) => {
          if !m.is_record() && !m.is_list() {
            Err(MalformedHeader(
              b,
              self.index - 1,
              "root header type must be either record or list",
            ))
          } else {
            Ok(Root(m))
          }
        }

        Some(b @ &Block::Header(..)) => Err(MalformedHeader(
          b,
          self.index - 1,
          "root header argument must be null",
        )),

        Some(b) => Err(UnexpectedBlock(b, self.index - 1, "expected header")),

        None => Ok(End),
      },

      Root(root_marker) => match self.consume_block() {
        Some(Block::Data(m, l, g)) => {
          if root_marker.is_record() && !m.is_field() {
            Err(WrongMarkerType(
              m,
              self.index,
              "data blocks in records must use a field marker",
            ))
          } else if root_marker.is_list() && !m.is_element() {
            Err(WrongMarkerType(
              m,
              self.index,
              "data blocks in lists must use a element marker",
            ))
          } else if l.0.decode::<u64>().unwrap() != g.width as u64 {
            Err(LengthMismatch(l, g, self.index))
          } else {
            Ok(State::Root(root_marker))
          }
        }

        Some(Block::Header(t, field)) => {
          if root_marker.is_record() && !field.is_field() {
            Err(WrongMarkerType(field, self.index, "expected a field maker"))
          } else if root_marker.is_list() && !field.is_element() {
            Err(WrongMarkerType(
              field,
              self.index,
              "expected element marker",
            ))
          } else {
            Ok(State::Nested(*t, Box::new(self.state.clone())))
          }
        }

        Some(Block::Terminator) => Err(UnexpectedTerminator(self.index)),

        None => Ok(End),
      },

      Nested(nested_marker, previous_state) => match self.consume_block() {
        Some(Block::Data(m, l, g)) => {
          if nested_marker.is_record() && !m.is_field() {
            Err(WrongMarkerType(
              m,
              self.index,
              "data blocks in records must use a field marker",
            ))
          } else if nested_marker.is_list() && !m.is_element() {
            Err(WrongMarkerType(
              m,
              self.index,
              "data blocks in lists must use a element marker",
            ))
          } else if l.0.decode::<u64>().unwrap() != g.width as u64 {
            Err(LengthMismatch(l, g, self.index))
          } else {
            Ok(State::Nested(nested_marker, previous_state))
          }
        }

        Some(Block::Header(t, field)) => {
          if nested_marker.is_record() && !field.is_field() {
            Err(WrongMarkerType(field, self.index, "expected a field maker"))
          } else if nested_marker.is_list() && !field.is_element() {
            Err(WrongMarkerType(
              field,
              self.index,
              "expected element marker",
            ))
          } else {
            Ok(State::Nested(*t, Box::new(self.state.clone())))
          }
        }

        Some(Block::Terminator) => Ok((*previous_state).clone()),

        None => Ok(End),
      },

      End => Ok(End),
    }
  }

  fn consume_block(&mut self) -> Option<&'a Block> {
    if self.index == self.obj.blocks.len() {
      return None;
    }
    let block = &self.obj.blocks[self.index];
    self.index += 1;
    Some(block)
  }
}

//! The `data` module defines the data layout of compressed objects.

use crate::bit::{BitVec, BitVecExt};
use crate::vie::CodePoint;

/// An interned identifier which can be mapped back to a named record field in
/// some schema.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FieldId(u32);

impl FieldId {
  pub fn new(i: u32) -> Self {
    FieldId(i)
  }
}

/// A section of a [Block] which denotes what field some piece of data belongs
/// to. Fields are encoded as fixed width integers where the width is determined
/// by the number of possible fields in a record.
///
/// [Block]: enum.Block.html
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Field {
  /// The number of bits that this field will take up once encoded.
  pub width: usize,
  /// The id of this field, or `None` if there is no associated id such as in
  /// the case of the root object.
  pub id: Option<FieldId>,
}

impl Field {
  /// Constructs a new `Field` with a given `width` and `id`.
  pub fn new(width: usize, id: FieldId) -> Self {
    Field {
      width,
      id: Some(id),
    }
  }

  /// Constructs a field with no `id` and a given width.
  pub fn null(width: usize) -> Self {
    Field { width, id: None }
  }
}

impl Into<BitVec> for Field {
  fn into(self) -> BitVec<u32> {
    let mut b = match self.id {
      None => BitVec::from_elem(self.width, false),
      Some(id) => BitVec::from_rev_be(id.0 + 1),
    };

    b.zext_or_trunc(self.width);
    b
  }
}

/// A section of a [Block] which denotes the length of a data section or list
/// object. Lengths are encoded using a variable width integer encoding similar
/// to UTF-8. See [CodePoint] for more information on their implementation.
///
/// [Block]: enum.Block.html
/// [CodePoint]: ../core/struct.CodePoint.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Length(usize);

impl Length {
  pub fn new(len: usize) -> Self {
    Length(len)
  }
}

impl Into<BitVec> for Length {
  fn into(self) -> BitVec<u32> {
    let codepoint = CodePoint::from(self.0 as u64);
    BitVec::from_bytes(codepoint.bytes())
  }
}

/// Blocks are the fundamental building block of compressed objects. Each
/// compressed object is just a sequence of blocks packed together in memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Block {
  /// A block which denotes the start of a record data object. Its [Field]
  /// component denotes what field the record belongs to. If a record is the
  /// root object or nested under a list, the field's `id` field will be
  /// `None`.
  ///
  /// [Field]: struct.Field.html
  RecordHeader(Field),

  /// A block which denotes the start of a list data object. It has two
  /// components, a [Field] and a [Length]. The field component determines what
  /// field this list is nested under if nested within a record object. The
  /// length component holds the number of elements in the list.
  ///
  /// [Field]: struct.Field.html
  /// [Length]: struct.Length.html
  ListHeader(Field, Length),

  /// A data block which contains encoded data for a single record field.
  ///
  /// Data held in this block has a fixed width which is determined from the
  /// schema.
  FixedWidthField(Field, BitVec),

  /// A data block which contains encoded data for a single record field.
  ///
  /// The data held in this type of block has a length which cannot be
  /// determined by the schema.
  VariableWidthField(Field, Length, BitVec),

  /// A data block which contains encoded data for a single list element.
  ///
  /// Data held in this type of block has a fixed width determined from the
  /// schema. Since lists must be homogeneous no length component is required
  /// when the element width can be statically determined.
  FixedWidthElement(BitVec),

  /// A data block which contains encoded data for a single list element.
  ///
  /// The length of the data held in this type of block cannot be determined by
  /// the schema so a length component is required.
  VariableWidthElement(Length, BitVec),

  /// The terminator block is used to mark the end of record objects.
  Terminator { width: usize },
}

impl std::fmt::Display for Block {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    use Block::*;

    let fmt_id = |m: &Field| {
      m.id
        .map_or_else(|| "None".to_string(), |id| format!("{}", id.0 + 1))
    };

    match self {
      RecordHeader(m) => write!(
        f,
        "HR  {{ width: {}, id: {} }}",
        m.width,
        fmt_id(m)
      ),
      ListHeader(m, l) => write!(
        f,
        "HL  {{ width: {}, id: {}, length: {} }}",
        m.width,
        fmt_id(m),
        l.0
      ),
      FixedWidthField(m, data) => write!(
        f,
        "FWF {{ width: {}, id: {}, data: {:?} }}",
        m.width,
        fmt_id(m),
        data
      ),
      VariableWidthField(m, l, data) => write!(
        f,
        "VWF {{ width: {}, id: {}, length: {}, data: {:?} }}",
        m.width,
        fmt_id(m),
        l.0,
        data
      ),
      FixedWidthElement(data) => {
        write!(f, "FixedWidthElement {{ data: {:?} }}", data.len())
      }
      VariableWidthElement(l, data) => write!(
        f,
        "VWE {{ length: {}, data: {:?} }}",
        l.0,
        data
      ),
      Terminator { width } => write!(f, "TER {{ width: {} }}", width),
    }
  }
}

impl Into<BitVec> for Block {
  fn into(self) -> BitVec<u32> {
    use Block::*;

    match self {
      RecordHeader(m) => m.into(),

      ListHeader(m, l) => {
        let mut b: BitVec = m.into();
        b.append(&mut l.into());
        b
      }

      FixedWidthField(m, mut data) => {
        let mut b: BitVec = m.into();
        b.append(&mut data);
        b
      }

      VariableWidthField(m, l, mut data) => {
        let mut b: BitVec = m.into();
        b.append(&mut l.into());
        b.append(&mut data);
        b
      }

      FixedWidthElement(data) => data,

      VariableWidthElement(l, mut data) => {
        let mut b: BitVec = l.into();
        b.append(&mut data);
        b
      }

      Terminator { width } => Field::null(width).into(),
    }
  }
}

/// A compressed object is simply a sequence of [Blocks] which represents some
/// structured data. When paired with a Schema it can be converted into a
/// human-readable representation like JSON.
///
/// [Blocks]: enum.Block.html
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressedObject {
  pub blocks: Vec<Block>,
}

impl CompressedObject {
  /// Constructs an empty compressed object with no blocks.
  pub fn new() -> Self {
    CompressedObject { blocks: Vec::new() }
  }

  /// Pushes a new block onto the end of this compressed object.
  pub fn push(&mut self, block: Block) {
    self.blocks.push(block);
  }
}

impl Default for CompressedObject {
  fn default() -> Self {
    Self::new()
  }
}

impl Into<BitVec> for CompressedObject {
  fn into(self) -> BitVec<u32> {
    let mut b = BitVec::new();
    for block in self.blocks {
      let mut bits: BitVec = block.into();
      b.append(&mut bits);
    }
    b
  }
}

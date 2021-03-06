//! The `schema` module implements the schema which is used to encode/decode
//! compressed objects.

use crate::data::FieldId;
use crate::math;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// The base type for a record field or list element.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Type {
  /// A special type which tells the schema that the data for this
  /// field/element should be encoded as-is without any special compression
  /// or encoding.
  ///
  /// This type can also be used as a fallback for data formats which are not
  /// supported by the program.
  PassThrough,

  /// A named type. The schema will parse and lookup this name and try and
  /// match it to a compression or encoding format that it knows about.
  Name(String),

  /// A nested record or list type.
  Nested(CompositeType),

  /// An enumeration of possible string values for this field/element.
  ///
  /// Since the schema knows about all possible values for this particular type
  /// it can efficiently encode them as integers which take up the minimum
  /// necessary number of bits.
  ///
  /// A `BTreeSet` is used here as a deterministic ordering on the variants is
  /// required. The schema uses the ordinal values of each variant when
  /// encoding.
  Enum {
    #[serde(rename = "enum")]
    variants: BTreeSet<String>,
  },
}

/// A composite type is either a record or list which is composed of other types
/// some of which may be other records or lists.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompositeType {
  Record(Record),
  List(List),
}

/// Record types are a mapping of field names to types.
///
/// A `BTreeMap` is required here because the fields in a record must have
/// a deterministic ordering. When encoding a [compressed object], a field's
/// ordinal value is used to uniquely identify the field in the record.
///
/// [compressed object]: ../data/struct.CompressedObject.html
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Record(pub BTreeMap<String, Type>);

impl Record {
  /// The width of field markers for this record type.
  pub fn field_width(&self) -> usize {
    math::required_bit_width(self.0.len() + 1)
  }

  /// A mapping of this record's field names to identifiers.
  pub fn field_map(&self) -> HashMap<&str, FieldId> {
    self
      .0
      .iter()
      .enumerate()
      .map(|(i, (k, _))| (k.as_str(), FieldId::new(i as u32)))
      .collect()
  }

  /// A mapping of identifiers to this record's field names.
  pub fn inverse_field_map(&self) -> HashMap<FieldId, &str> {
    self
      .0
      .iter()
      .enumerate()
      .map(|(i, (k, _))| (FieldId::new(i as u32), k.as_str()))
      .collect()
  }
}

/// Lists are a repetition of many values with a single type.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct List(pub Box<Type>);

/// The schema acts as a type definition for some structured data. It tells the
/// program how each field/element should be encoded and acts as a lookup table
/// when constructing and deconstructing [compressed objects].
///
/// [compressed objects]: ../data/struct.CompressedObject.html
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Schema(CompositeType);

impl Schema {
  /// Constructs a new schema.
  pub fn new(root: CompositeType) -> Self {
    Schema(root)
  }

  /// The root type of this schema.
  #[inline]
  pub fn root(&self) -> &CompositeType {
    &self.0
  }
}

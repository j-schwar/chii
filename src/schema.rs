//! This module defines the symbolic structure of the static schema used to
//! serialize and deserialize compressed objects.

use crate::compress;
use crate::compress::{Compressor, EnumCompressor, PassThroughCompressor};
use crate::core::math;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// The number of reserved marker values, also referred to as keys.
const NUM_RESERVED_KEYS: usize = 4;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Schema {
  Record(BTreeMap<String, Type>),
  List(Box<Type>),
}

impl Schema {
  /// Constructs a new `Record` schema.
  pub fn new_record<S, I: IntoIterator<Item = (S, Type)>>(i: I) -> Self
  where
    S: Into<String>,
  {
    let btree: BTreeMap<String, Type> =
      i.into_iter().map(|(k, v)| (k.into(), v)).collect();
    Schema::Record(btree)
  }

  /// Constructs a new `List` schema.
  pub fn new_list(t: Type) -> Self {
    Schema::List(Box::new(t))
  }

  /// The minimum width of markers.
  pub fn marker_width(&self) -> usize {
    // TODO: consider what happens when we have nested objects.
    let count = match self {
      Schema::Record(btree) => btree.len() + NUM_RESERVED_KEYS,
      Schema::List(_) => NUM_RESERVED_KEYS,
    };
    math::required_bit_width(count)
  }

  /// A mapping of field names to marker values.
  pub fn field_map(&self) -> HashMap<&str, usize> {
    if let Schema::Record(btree) = self {
      btree
        .keys()
        .enumerate()
        .map(|(i, k)| (k.as_str(), i + NUM_RESERVED_KEYS))
        .collect()
    } else {
      panic!("key_map is only defined for Record schemas");
    }
  }

  /// A mapping of marker values to field names.
  pub fn inverse_field_map(&self) -> HashMap<usize, &str> {
    if let Schema::Record(btree) = self {
      btree
        .keys()
        .enumerate()
        .map(|(i, k)| (i + NUM_RESERVED_KEYS, k.as_str()))
        .collect()
    } else {
      panic!("inverse_key_map is only defined for Record schemas");
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Type {
  /// Leave this field/element untouched and pass it through to the compressed
  /// object as is.
  PassThrough,

  /// A builtin type native to the compressor.
  ///
  /// This includes your standard types like `u8`, `i32`, `bool`, etc. but also
  /// includes some custom types like `uuid` for Universally Unique Identifiers,
  /// or `ascii` for ASCII encoded text.
  Builtin(String),

  /// A recursive schema type used for nested records/lists.
  Schema(Box<Schema>),

  /// An enumeration type.
  ///
  /// Values of this type must be strings and their value must exist in the set
  /// of valid enum values.
  Enum {
    /// How the enumeration is processed.
    mode: EnumMode,

    /// The possible variants of the enumeration.
    variants: Vec<String>,
  },
}

impl Type {
  /// Constructs a new builtin type.
  pub fn new_builtin<S: Into<String>>(name: S) -> Self {
    Type::Builtin(name.into())
  }

  /// Constructs a new schema type.
  pub fn new_schema(schema: Schema) -> Self {
    Type::Schema(schema.into())
  }

  /// Constructs a new enum type.
  pub fn new_enum<S: Into<String>>(mode: EnumMode, variants: Vec<S>) -> Self {
    Type::Enum {
      mode,
      variants: variants.into_iter().map(|x| x.into()).collect(),
    }
  }

  /// True if this is an integer type.
  ///
  /// Integer types expect a `u64` and input and produce a `u64` as output.
  pub fn is_integer_type(&self) -> bool {
    match self {
      Type::Builtin(name) => {
        name.starts_with('u') && name[1..].chars().all(|c| c.is_ascii_digit())
      }
      _ => false,
    }
  }

  /// True if this is a boolean type.
  pub fn is_bool_type(&self) -> bool {
    match self {
      Type::Builtin(name) if name == "bool" => true,
      _ => false,
    }
  }

  /// True if this type has a statically known encoded width.
  pub fn is_fixed_width(&self) -> bool {
    // TODO: This is a hack, find a better way to do this.
    if self.is_integer_type() || self.is_bool_type() {
      return true;
    }

    match self {
      Type::Builtin(n) => n == "uuid",
      _ => false,
    }
  }

  /// Returns a compressor for this type.
  ///
  /// Returns `None` if unable to construct the required compressor or if the
  /// type is a `Schema` type.
  pub fn compressor(&self) -> Option<Box<dyn Compressor>> {
    use Type::*;
    match self {
      PassThrough => Some(Box::new(PassThroughCompressor)),

      Builtin(name) => compress::builtin(name),

      Schema(_) => None,

      Enum { variants, mode } => {
        if *mode != EnumMode::Strict {
          panic!("only strict enum mode is supported at the moment");
        }
        Some(Box::new(EnumCompressor::from_string_variants(variants)))
      }
    }
  }
}

/// Describes how enumeration values should be encoded/decoded.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EnumMode {
  /// Variants are matched as is (i.e., case sensitive).
  Strict,

  /// Variants have their case normalized before matching.
  Caseless,
}

use std::convert::TryFrom;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use crate::comp::{self, Compressor, EncodedWidth};
use crate::data::{Block, CompressedObject, Field, Length};
use crate::schema::{CompositeType, List, Record, Schema, Type};

/// Encodes a JSON `value` using a given `schema`.
pub fn encode(schema: &Schema, value: &Value) -> Result<CompressedObject> {
  let mut co = CompressedObject::new();
  encode_composite_type(schema.root(), None, &mut co, value)?;
  Ok(co)
}

/// Encodes a composite type.
fn encode_composite_type(
  ct: &CompositeType,
  field: Option<Field>,
  co: &mut CompressedObject,
  value: &Value,
) -> Result<()> {
  match ct {
    CompositeType::Record(r) => encode_record(&r, field, co, value),
    CompositeType::List(l) => encode_list(&l, field, co, value),
  }
}

/// Encodes a list type.
fn encode_list(
  list: &List,
  field: Option<Field>,
  co: &mut CompressedObject,
  value: &Value,
) -> Result<()> {
  // Cast `value` into an array first as we need its length for the header
  let arr = value.as_array().ok_or_else(|| anyhow!("expected array"))?;

  // If this list is nested push its header on first
  if let Some(f) = field {
    let len = Length::new(arr.len());
    let header = Block::ListHeader(f, len);
    co.push(header);
  }

  // Encode each element in the list
  for v in arr {
    if let Type::Nested(ct) = list.0.as_ref() {
      encode_composite_type(ct, None, co, v)
    } else {
      encode_element(list.0.as_ref(), co, v)
    }
    .with_context(|| "when encoding list element")?;
  }

  Ok(())
}

/// Encodes a record type.
fn encode_record(
  record: &Record,
  field: Option<Field>,
  co: &mut CompressedObject,
  value: &Value,
) -> Result<()> {
  // If this record is nested, push its header on first
  if let Some(f) = field {
    let header = Block::RecordHeader(f);
    co.push(header);
  }

  // Cast `value` into an object
  let value_map = value
    .as_object()
    .ok_or_else(|| anyhow!("expected object"))?;

  // Compute the mapping of field names to identifiers and figure out the field
  // width for this record's elements
  let field_map = record.field_map();
  let field_width = record.field_width();

  // Encode each field as they appear in the value object
  for (k, v) in value_map {
    let id = field_map
      .get(k.as_str())
      .ok_or_else(|| anyhow!("unexpected field: {}", k))?;
    let field = Field::new(field_width, *id);
    let ty = &record.0[k];

    // If the expected type for a field is a nested type (i.e., record or list)
    // recurse and try an encode the composite type. Note that we switch based
    // on the expected type as defined in the schema and not what the value
    // actually is. The schema is what drives the encoding process, not the
    // value.
    //
    // If not, then we just encode the value normally.
    if let Type::Nested(ct) = ty {
      encode_composite_type(ct, Some(field), co, v)
    } else {
      encode_field(field, ty, co, v)
    }
    .with_context(|| format!("when encoding {}", k))?;
  }

  // Push the terminator block if this is a nested record
  if field.is_some() {
    // Terminator uses the same field width as the rest of this record's fields
    co.push(Block::Terminator { width: field_width });
  }

  Ok(())
}

/// Encodes a non-nested element.
fn encode_element(
  ty: &Type,
  co: &mut CompressedObject,
  value: &Value,
) -> Result<()> {
  let compressor = get_compressor_for_type(ty)?;
  let value = comp::Value::try_from(value)?;
  let bits = compressor.compress(value)?;

  let block = if compressor.encoded_width() == EncodedWidth::Variable {
    let len = Length::new(bits.len());
    Block::VariableWidthElement(len, bits)
  } else {
    Block::FixedWidthElement(bits)
  };

  co.push(block);
  Ok(())
}

/// Encodes a non-nested field.
fn encode_field(
  field: Field,
  ty: &Type,
  co: &mut CompressedObject,
  value: &Value,
) -> Result<()> {
  let compressor = get_compressor_for_type(ty)?;
  let value = comp::Value::try_from(value)?;
  let bits = compressor.compress(value)?;

  let block = if compressor.encoded_width() == EncodedWidth::Variable {
    let len = Length::new(bits.len());
    Block::VariableWidthField(field, len, bits)
  } else {
    Block::FixedWidthField(field, bits)
  };

  co.push(block);
  Ok(())
}

fn get_compressor_for_type(ty: &Type) -> Result<Box<dyn Compressor>> {
  use Type::*;

  match ty {
    PassThrough => Ok(Box::new(comp::IdentityCompressor)),
    Name(name) => lookup_named_compressor(name),
    Enum { variants } => Ok(Box::new(comp::EnumCompressor {
      variants: variants.iter().cloned().collect(),
    })),
    Nested(_) => panic!("cannot get compressor for composite type"),
  }
}

/// Attempts to find the compressor for a given name. Returns `None` if unable
/// to find a compressor.
fn lookup_named_compressor(name: &str) -> Result<Box<dyn Compressor>> {
  match name {
    "bool" => Ok(Box::new(comp::BooleanCompressor)),
    _ => bail!("cannot determine compressor for '{}'", name),
  }
}

//! Functions for converting data to and from compressed objects.

use crate::compress::Compressor;
use crate::prelude::*;
use serde_json::{Map, Value as JsonValue};
use std::error::Error;

#[derive(Debug)]
pub enum TranscodeError {
  UnexpectedField(String),
  WrongFieldType(&'static str),
  UnknownBuiltinName(String),
  FailedToGetCompressor(String),
  CompressionError(Box<dyn Error>),
  WrongValueType,
}

/// Converts a JSON value into a compressed object following a given schema.
pub fn from_json(
  json: &JsonValue,
  schema: &Schema,
) -> Result<CompressedObject, TranscodeError> {
  if let Some(object) = json.as_object() {
    let mut co = CompressedObject::new_record();
    compress_object(&mut co, object, schema)?;
    Ok(co)
  } else if let Some(_array) = json.as_array() {
    unimplemented!()
  } else {
    Err(TranscodeError::WrongValueType)
  }
}

fn compress_object(
  co: &mut CompressedObject,
  object: &Map<String, JsonValue>,
  schema: &Schema,
) -> Result<(), TranscodeError> {
  use TranscodeError::*;
  if let Schema::Record(fields) = &schema {
    let field_map = schema.field_map();
    for (key, value) in object {
      // Find the type for this field from the schema.
      let t = fields
        .get(key)
        .ok_or_else(|| UnexpectedField(key.clone()))?;

      // If the type is another schema then we expect a nested object/list.
      if let Type::Schema(_nested) = t {
        unimplemented!("nested objects have not yet been implemented");
      }

      // Get the compressor for this field.
      let compressor = t
        .compressor()
        .ok_or_else(|| FailedToGetCompressor(key.clone()))?;

      // Compress the value.
      let glob = if t.is_integer_type() {
        compress_integer_value(value, compressor.as_ref())?
      } else {
        // Assume that the value is a string.
        // TODO: Handle other types of values.
        let s = value.as_str().expect("expected a string value");
        compressor
          .compress(s.as_bytes())
          .map_err(|e| CompressionError(e))?
      };

      // Convert key to marker and push onto compressed object.
      let marker = Marker::Field(field_map[key.as_str()] as u32);
      co.push_data(marker, glob);
    }
    Ok(())
  } else {
    Err(TranscodeError::WrongValueType)
  }
}

/// Compresses a JSON integer value into a glob.
fn compress_integer_value(
  json: &JsonValue,
  compressor: &dyn Compressor,
) -> Result<Glob, TranscodeError> {
  let value = json
    .as_u64()
    .ok_or_else(|| TranscodeError::WrongFieldType("expected integer"))?;
  compressor
    .compress(&value.to_le_bytes())
    .map_err(|e| TranscodeError::CompressionError(e))
}
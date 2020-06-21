use super::*;

#[test]
fn valid_linear_record() {
  let mut co = CompressedObject::new_record();
  co.push_data(Marker::Field(4), Glob::new(16, vec![1, 2]));
  co.push_data(Marker::Field(5), Glob::new(21, vec![3, 4, 5]));
  assert!(co.validate().is_ok());
}

#[test]
fn valid_linear_list() {
  let mut co = CompressedObject::new_list();
  co.push_data(Marker::Element, Glob::new(16, vec![1, 2]));
  co.push_data(Marker::Element, Glob::new(21, vec![3, 4, 5]));
  assert!(co.validate().is_ok());
}

#[test]
fn invalid_linear_record_element_marker_for_field_name() {
  let mut co = CompressedObject::new_record();
  co.push_data(Marker::Element, Glob::new(16, vec![1, 2]));
  assert!(co.validate().is_err());
}

#[test]
fn invalid_linear_list_field_marker_for_element() {
  let mut co = CompressedObject::new_list();
  co.push_data(Marker::Field(5), Glob::new(8, vec![1]));
  assert!(co.validate().is_err());
}

#[test]
fn invalid_linear_record_unexpected_terminator() {
  let mut co = CompressedObject::new_record();
  co.push(Block::Terminator);
  assert!(co.validate().is_err());
}

#[test]
fn invalid_linear_list_unexpected_terminator() {
  let mut co = CompressedObject::new_list();
  co.push(Block::Terminator);
  assert!(co.validate().is_err());
}

#[test]
fn valid_nested_object() {
  let mut co = CompressedObject::new_record();
  co.begin_nested_record(Marker::Field(4));
  co.push_data(Marker::Field(6), Glob::new(8, vec![1]));
  co.push_data(Marker::Field(7), Glob::new(8, vec![2]));

  co.begin_nested_record(Marker::Field(4));
  co.push_data(Marker::Field(6), Glob::new(8, vec![1]));
  co.push_data(Marker::Field(7), Glob::new(8, vec![2]));
  co.end_nested_object();

  co.end_nested_object();

  co.begin_nested_list(Marker::Field(5));
  co.push_data(Marker::Element, Glob::new(8, vec![3]));
  co.end_nested_object();

  co.push_data(Marker::Field(8), Glob::new(8, vec![4]));

  assert!(co.validate().is_ok());
}

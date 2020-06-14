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

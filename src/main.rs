use dsc::schema::*;
use dsc::transcode;
use serde_json::Value;

fn main() {
  let schema = Schema::new_record(vec![
    ("temp_high", Type::new_builtin("u7")),
    ("temp_low", Type::new_builtin("u7")),
    ("precipitation_probability", Type::new_builtin("u7")),
    ("precipitation", Type::new_builtin("u10")),
    ("pressure", Type::new_builtin("u12")),
    (
      "uv",
      Type::new_enum(
        EnumMode::Strict,
        vec!["HIGHEST", "HIGH", "AVERAGE", "LOW", "LOWEST"],
      ),
    ),
    (
      "weather",
      Type::new_enum(
        EnumMode::Strict,
        vec!["SUNNY", "RAINING", "PARTLY_CLOUDY", "SNOWING"],
      ),
    ),
  ]);
  let y = serde_yaml::to_string(&schema).unwrap();
  println!("--- Schema ---\n{}\n", y);

  let json_string =
    std::fs::read_to_string("./weather.json").expect("couldn't read file");
  let json = serde_json::from_str::<Value>(&json_string).expect("failed to parse json");
  println!("json {:3} bytes", json.to_string().bytes().len());

  let result = transcode::from_json(&json, &schema);
  match result {
    Ok(co) => {
      let bytes = co.into_bytes(schema.marker_width());
      println!("  co {:3} bytes: {:02x?}", bytes.len(), bytes);
    }
    Err(e) => println!("{:?}", e),
  }
}

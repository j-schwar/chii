use dsc::schema::*;
use dsc::transcode;
use serde_json::Value;

fn main() {
  let schema_string =
    std::fs::read_to_string("./samples/schema.yaml").expect("couldn't read schema");
  let schema =
    serde_yaml::from_str::<Schema>(&schema_string).expect("failed to parse schema");

  let json_string =
    std::fs::read_to_string("./samples/referral.json").expect("couldn't read file");
  let json = serde_json::from_str::<Value>(&json_string).expect("failed to parse json");
  println!("json {:3} bytes", json.to_string().bytes().len());

  let result = transcode::from_json(&json, &schema);
  match result {
    Ok(co) => {
      let bytes = co.into_bytes(schema.marker_width());
      println!("  co {:3} bytes", bytes.len());
    }
    Err(e) => println!("{:?}", e),
  }
}

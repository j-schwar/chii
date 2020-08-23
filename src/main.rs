use chii::schema::*;

fn main() {
  let schema = Schema::new(CompositeType::Record(
    vec![
      ("name".to_string(), Type::Name("ascii".to_string())),
      ("age".to_string(), Type::Name("0..120".to_string())),
      (
        "courses".to_string(),
        Type::Nested(CompositeType::List(Box::new(Type::Nested(
          CompositeType::Record(
            vec![
              ("name".to_string(), Type::Name("ascii".to_string())),
              ("credits".to_string(), Type::Name("3..4".to_string())),
              (
                "grade".to_string(),
                Type::Enum {
                  variants: vec!["A", "B", "C", "D", "E"]
                    .into_iter()
                    .map(|x| x.into())
                    .collect(),
                },
              ),
            ]
            .into_iter()
            .collect(),
          ),
        )))),
      ),
    ]
    .into_iter()
    .collect(),
  ));

  let s = serde_yaml::to_string(&schema).unwrap();
  println!("{}", s);
}

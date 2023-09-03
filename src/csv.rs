use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Int(i64),
  Float(f64),
  String(String),
}

pub fn read_csv(path: &str) -> Vec<Vec<Value>> {
  let text = fs::read_to_string(path).unwrap();
  let mut rows = Vec::<Vec<Value>>::new();
  for line in text.lines() {
    let mut values = Vec::<Value>::new();
    for value in line.split(',') {
      let value = value.trim();
      if let Ok(int) = value.parse::<i64>() {
        values.push(Value::Int(int));
      } else if let Ok(float) = value.parse::<f64>() {
        values.push(Value::Float(float));
      } else {
        values.push(Value::String(value.to_string()));
      }
    }
    rows.push(values);
  }
  rows
}

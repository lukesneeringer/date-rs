use std::fmt;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Visitor;

use crate::Date;

impl Serialize for Date {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&self.format("%Y-%m-%d"))
  }
}

struct DateVisitor;

impl Visitor<'_> for DateVisitor {
  type Value = Date;

  #[cfg(not(tarpaulin_include))]
  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a YYYY-MM-DD date string")
  }

  fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
    s.parse().map_err(E::custom)
  }
}

impl<'de> Deserialize<'de> for Date {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    deserializer.deserialize_str(DateVisitor)
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use super::*;

  #[test]
  fn test_serde() -> Result<(), serde_json::Error> {
    let json = r#"{"date":"2012-04-21"}"#;
    let struct_: TestStruct = serde_json::from_str(json)?;
    check!(struct_.date == date! { 2012-04-21 });
    let json = serde_json::to_string(&struct_)?;
    check!(json == r#"{"date":"2012-04-21"}"#);
    Ok(())
  }

  #[derive(Deserialize, Serialize)]
  struct TestStruct {
    date: Date,
  }
}

//! Integration with DuckDB.

use duckdb::Result;
use duckdb::types::FromSql;
use duckdb::types::FromSqlError;
use duckdb::types::FromSqlResult;
use duckdb::types::ToSql;
use duckdb::types::ToSqlOutput;
use duckdb::types::ValueRef;

use crate::Date;

impl FromSql for Date {
  fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
    match value {
      ValueRef::Date32(d) => Ok(Self(d)),
      _ => Err(FromSqlError::InvalidType),
    }
  }
}

impl ToSql for Date {
  fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
    Ok(ToSqlOutput::Borrowed(ValueRef::Date32(self.0)))
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use super::*;

  #[test]
  fn test_to_sql() -> Result<()> {
    let dt = date! { 2012-04-21 };
    let output = dt.to_sql()?;
    if let ToSqlOutput::Borrowed(ValueRef::Date32(i)) = output {
      check!(i == 15_451);
    } else {
      check!(false, "Incorrect type");
    }
    Ok(())
  }
}

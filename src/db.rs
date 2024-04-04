//! Serialization to/from PostgreSQL

use diesel::deserialize::FromSql;
use diesel::deserialize::Result as DeserializeResult;
use diesel::pg::data_types::PgDate;
use diesel::pg::Pg;
use diesel::pg::PgValue;
use diesel::serialize::Output;
use diesel::serialize::Result as SerializeResult;
use diesel::serialize::ToSql;
use diesel::sql_types;

use crate::Date;
use crate::DateInterval;

impl ToSql<sql_types::Date, Pg> for Date {
  fn to_sql<'se>(&'se self, out: &mut Output<'se, '_, Pg>) -> SerializeResult {
    let days_since_epoch = (*self - PG_EPOCH).days();
    ToSql::<sql_types::Date, Pg>::to_sql(&PgDate(days_since_epoch), &mut out.reborrow())
  }
}

impl FromSql<sql_types::Date, Pg> for Date {
  fn from_sql(bytes: PgValue<'_>) -> DeserializeResult<Self> {
    let PgDate(offset) = FromSql::<diesel::sql_types::Date, Pg>::from_sql(bytes)?;
    let duration = DateInterval::new(offset);
    Ok(PG_EPOCH + duration)
  }
}

const PG_EPOCH: Date = date! { 2000-01-01 };

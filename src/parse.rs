use std::fmt;
use std::str::FromStr;

use crate::Date;

impl FromStr for Date {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    macro_rules! fail {
      ($s:ident, $r:literal) => {
        ParseError { src: $s.into(), reason: Some($r) }
      };
    }
    macro_rules! assert {
      ($s:ident, $e:expr, $r:literal) => {
        if !($e) {
          Err(fail!($s, $r))?;
        }
      };
    }
    let pieces: Vec<&str> = s.split('-').collect();
    assert!(s, pieces.len() == 3, "Too many components in date.");
    assert!(s, pieces[0].len() == 4, "Invalid year length.");
    assert!(s, pieces[1].len() == 2, "Invalid month length.");
    assert!(s, pieces[2].len() == 2, "Invalid day length.");
    let year = pieces[0].parse::<i16>().map_err(|_| fail!(s, "Failed to parse year"))?;
    let month = pieces[1].parse::<u8>().map_err(|_| fail!(s, "Failed to parse month"))?;
    let day = pieces[2].parse::<u8>().map_err(|_| fail!(s, "Failed to parse day"))?;
    Ok(Date::new(year, month, day))
  }
}

#[derive(Debug)]
pub struct ParseError {
  src: String,
  reason: Option<&'static str>,
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Parse error attempting to parse Date from {}{}",
      self.src,
      self.reason.map(|r| format!(": {}", r)).unwrap_or_default(),
    )
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use super::*;

  #[test]
  fn test_parse() -> Result<(), ParseError> {
    check!("2012-04-21".parse::<Date>()? == date! { 2012-04-21 });
    check!("2012-4-21".parse::<Date>().is_err());
    check!("04/21/2012".parse::<Date>().is_err());
    check!("12-04-21".parse::<Date>().is_err());
    check!("foo".parse::<Date>().map_err(|e| e.to_string()).unwrap_err().contains("foo"));
    Ok(())
  }
}

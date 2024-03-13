use std::fmt::Display;
use std::fmt::Write;

use crate::Date;

impl Date {
  /// Format the date according to the provided `strftime` specifier.
  pub fn format<'a>(&'a self, format_str: &'a str) -> FormattedDate {
    FormattedDate { date: self, format: format_str }
  }
}

/// A date with a requested format.
#[derive(Debug)]
pub struct FormattedDate<'a> {
  date: &'a Date,
  format: &'a str,
}

impl<'a> Display for FormattedDate<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // Iterate over the format string and consume it.
    let d = self.date;
    let mut flag = false;
    for c in self.format.chars() {
      if flag {
        flag = false;
        match c {
          'Y' => write!(f, "{}", d.year)?,
          'C' => write!(f, "{:02}", d.year / 100)?,
          'y' => write!(f, "{:02}", d.year % 100)?,
          'm' => write!(f, "{:02}", d.month())?,
          'b' | 'h' => write!(f, "{}", d.month_abbv())?,
          'B' => write!(f, "{}", d.month_name())?,
          'd' => write!(f, "{:02}", d.day())?,
          'e' => write!(f, "{:2}", d.day())?,
          'a' => write!(f, "{}", d.weekday().abbv())?,
          'A' => write!(f, "{}", d.weekday())?,
          'w' => write!(f, "{}", d.weekday() as u8)?,
          'u' => write!(f, "{}", match d.weekday() {
            crate::Weekday::Sunday => 7,
            _ => self.date.weekday() as u8,
          })?,
          // U, W
          'j' => write!(f, "{:03}", d.day_of_year_0 + 1)?,
          'D' => write!(f, "{:02}/{:02}/{:02}", d.month(), d.day(), d.year)?,
          'F' => write!(f, "{:04}-{:02}-{:02}", d.year, d.month(), d.day())?,
          'v' => write!(f, "{:2}-{}-{:04}", d.day(), d.month_abbv(), d.year())?,
          't' => f.write_char('\t')?,
          'n' => f.write_char('\n')?,
          '%' => f.write_char('%')?,
          _ => Err(std::fmt::Error)?,
        }
      }
      else if c == '%' {
        flag = true;
      }
      else {
        f.write_char(c)?;
      }
    }
    Ok(())
  }
}

impl<'a> PartialEq<&str> for FormattedDate<'a> {
  fn eq(&self, other: &&str) -> bool {
    &self.to_string().as_str() == other
  }
}

macro_rules! month_str {
  ($($num:literal => $short:ident ~ $long:ident)*) => {
    impl Date {
      /// The English name of the month.
      const fn month_name(&self) -> &'static str {
        match self.month() {
          $($num => stringify!($long),)*
          _ => panic!("Fictitious month"),
        }
      }

      /// The three-letter abbreviation of the month.
      const fn month_abbv(&self) -> &'static str {
        match self.month() {
          $($num => stringify!($short),)*
          _ => panic!("Fictitious month"),
        }
      }
    }
  }
}
month_str! {
   1 => Jan ~ January
   2 => Feb ~ February
   3 => Mar ~ March
   4 => Apr ~ April
   5 => May ~ May
   6 => Jun ~ June
   7 => Jul ~ July
   8 => Aug ~ August
   9 => Sep ~ September
  10 => Oct ~ October
  11 => Nov ~ November
  12 => Dec ~ December
}

#[cfg(test)]
mod tests {
  use assert2::check;

  #[test]
  fn test_format() {
    let date = date! { 2012-04-21 };
    for (fmt_string, date_str) in [
      ("%Y-%m-%d", "2012-04-21"),
      ("%B %e, %Y", "April 21, 2012"),
      ("%A, %B %e, %Y", "Saturday, April 21, 2012"),
      ("%e %h %Y", "21 Apr 2012"),
      ("%a %e %b %Y", "Sat 21 Apr 2012"),
      ("%m/%d/%y", "04/21/12"),
    ] {
      check!(date.format(fmt_string).to_string() == date_str);
    }
  }
}

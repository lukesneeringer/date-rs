use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Write;

use crate::Date;

/// A date with a requested format.
pub struct FormattedDate<'a> {
  pub(crate) date: Date,
  pub(crate) format: &'a str,
}

impl Debug for FormattedDate<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    Display::fmt(self, f)
  }
}

impl Display for FormattedDate<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    // Iterate over the format string and consume it.
    let d = self.date;
    let ymd = self.date.ymd();
    let mut flag = false;
    let mut padding = Padding::Default;
    for c in self.format.chars() {
      if flag {
        // Apply padding if this is a padding change.
        #[rustfmt::skip]
        match c {
          '0' => { padding = Padding::Zero; continue; },
          '-' => { padding = Padding::Suppress; continue; },
          '_' => { padding = Padding::Space; continue; },
          _ => {},
        };

        // Set up a macro to process padding.
        macro_rules! write_padded {
          ($f:ident, $pad:ident, $level:literal, $e:expr) => {
            match $pad {
              Padding::Default | Padding::Zero => write!($f, concat!("{:0", $level, "}"), $e),
              Padding::Space => write!($f, concat!("{:", $level, "}"), $e),
              Padding::Suppress => write!($f, "{}", $e),
            }
          };
        }

        // Write out the formatted component.
        flag = false;
        match c {
          'Y' => write_padded!(f, padding, 4, ymd.0)?,
          'C' => write_padded!(f, padding, 2, ymd.0 / 100)?,
          'y' => write_padded!(f, padding, 2, ymd.0 % 100)?,
          'm' => write_padded!(f, padding, 2, ymd.1)?,
          'b' | 'h' => write!(f, "{}", d.month_abbv())?,
          'B' => write!(f, "{}", d.month_name())?,
          'd' => write_padded!(f, padding, 2, ymd.2)?,
          'a' => write!(f, "{}", d.weekday().abbv())?,
          'A' => write!(f, "{}", d.weekday())?,
          'w' => write!(f, "{}", d.weekday() as u8)?,
          'u' => write!(f, "{}", match d.weekday() {
            crate::Weekday::Sunday => 7,
            _ => self.date.weekday() as u8,
          })?,
          // U, W
          'j' => write_padded!(f, padding, 3, d.day_of_year())?,
          'U' => write_padded!(f, padding, 2, d.week())?,
          'D' => write!(f, "{:02}/{:02}/{:02}", ymd.1, ymd.2, ymd.0)?,
          'F' => write!(f, "{:04}-{:02}-{:02}", ymd.0, ymd.1, ymd.2)?,
          'v' => write!(f, "{:2}-{}-{:04}", d.day(), d.month_abbv(), d.year())?,
          't' => f.write_char('\t')?,
          'n' => f.write_char('\n')?,
          '%' => f.write_char('%')?,
          _ => Err(Error)?,
        }
      } else if c == '%' {
        flag = true;
        padding = Padding::Default;
      } else {
        f.write_char(c)?;
      }
    }
    Ok(())
  }
}

impl PartialEq<&str> for FormattedDate<'_> {
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
          #[cfg(not(tarpaulin_include))]
          _ => panic!("Fictitious month"),
        }
      }

      /// The three-letter abbreviation of the month.
      const fn month_abbv(&self) -> &'static str {
        match self.month() {
          $($num => stringify!($short),)*
          #[cfg(not(tarpaulin_include))]
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

/// A padding modifier
enum Padding {
  /// Use the default padding (usually either `0` or nothing).
  Default,
  /// Explicitly pad with `0`
  Zero,
  /// Explicitly pad with ` `.
  Space,
  /// Explicitly prevent padding, even if the token has default padding.
  Suppress,
}

#[cfg(test)]
mod tests {
  use assert2::check;

  #[test]
  fn test_format() {
    let date = date! { 2012-04-21 };
    for (fmt_string, date_str) in [
      ("%Y-%m-%d", "2012-04-21"),
      ("%F", "2012-04-21"),
      ("%v", "21-Apr-2012"),
      ("%B %-d, %Y", "April 21, 2012"),
      ("%B %-d, %C%y", "April 21, 2012"),
      ("%A, %B %-d, %Y", "Saturday, April 21, 2012"),
      ("%d %h %Y", "21 Apr 2012"),
      ("%a %d %b %Y", "Sat 21 Apr 2012"),
      ("%m/%d/%y", "04/21/12"),
      ("year: %Y / day: %j", "year: 2012 / day: 112"),
      ("%%", "%"),
      ("%w %u", "6 6"),
      ("%t %n", "\t \n"),
      ("%Y week %U", "2012 week 16"),
    ] {
      check!(date.format(fmt_string).to_string() == date_str);
      check!(date.format(fmt_string) == date_str);
      check!(format!("{:?}", date.format(fmt_string)) == date_str);
    }
  }

  #[test]
  fn test_padding() {
    let date = date! { 2024-07-04 };
    for (fmt_string, date_str) in
      [("%Y-%m-%d", "2024-07-04"), ("%B %-d, %Y", "July 4, 2024"), ("%-d-%h-%Y", "4-Jul-2024")]
    {
      check!(date.format(fmt_string).to_string() == date_str);
      check!(date.format(fmt_string) == date_str);
    }
  }
}

use std::fmt::Display;

use crate::Date;

impl Date {
  /// Return the weekday corresponding to the given date.
  ///
  /// Implementation for this explained at [Art of Memory][aom].
  ///
  /// [aom]: https://artofmemory.com/blog/how-to-calculate-the-day-of-the-week/
  #[inline]
  pub const fn weekday(&self) -> Weekday {
    let year_abs = self.year().unsigned_abs();
    let year_code = (year_abs % 100 + (year_abs % 100 / 4)) % 7;

    // Note: These values are offset by one from the referenced website because
    // we are using 0-offset days of the year, rather than 1-offset days of the
    // month plus month codes (as the website recommends).
    //
    // We follow this instead of the website's approach because (1) it better
    // fits our data model and (2) it removes the need for month codes at all.
    let century_code = match self.year() / 100 % 4 {
      0 => 7,
      1 => 5,
      2 => 3,
      3 => 1,
      #[cfg(not(tarpaulin_include))]
      _ => panic!("Unreachable: n % 4 is always within `0..=4`."),
    };
    let leap_year = match self.is_leap_year() {
      true => 1,
      false => 0,
    };
    match (year_code + century_code + self.day_of_year_0 - leap_year) % 7 {
      0 => Weekday::Sunday,
      1 => Weekday::Monday,
      2 => Weekday::Tuesday,
      3 => Weekday::Wednesday,
      4 => Weekday::Thursday,
      5 => Weekday::Friday,
      6 => Weekday::Saturday,
      #[cfg(not(tarpaulin_include))]
      _ => panic!("Unreachable: Fake weekday"),
    }
  }
}

/// A representation of the day of the week.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Weekday {
  Sunday = 0,
  Monday = 1,
  Tuesday = 2,
  Wednesday = 3,
  Thursday = 4,
  Friday = 5,
  Saturday = 6,
}

impl Display for Weekday {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    macro_rules! display {
      ($($e:ident),*) => {
        f.write_str(match self {
          $(Self::$e => stringify!($e)),*
        })
      };
    }
    display!(Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday)
  }
}

impl Weekday {
  /// The three-letter abbreviation for this weekday.
  pub(crate) fn abbv(&self) -> &'static str {
    match self {
      Self::Sunday => "Sun",
      Self::Monday => "Mon",
      Self::Tuesday => "Tue",
      Self::Wednesday => "Wed",
      Self::Thursday => "Thu",
      Self::Friday => "Fri",
      Self::Saturday => "Sat",
    }
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use super::*;
  use crate::Duration;

  #[test]
  fn test_weekday() {
    let mut date = date! { 2019-12-29 }; // A Sunday.
    for weekday in [
      Weekday::Sunday,
      Weekday::Monday,
      Weekday::Tuesday,
      Weekday::Wednesday,
      Weekday::Thursday,
      Weekday::Friday,
      Weekday::Saturday,
    ]
    .into_iter()
    .cycle()
    {
      check!(date.weekday() == weekday, "Incorrect on: {:?}", date);
      date += Duration::days(1);
      if date.year() == 2022 && date.month() == 2 {
        break;
      }
    }

    // Also check some random dates in other centuries.
    check!(date! { 1700-01-01 }.weekday() == Weekday::Friday);
    check!(date! { 1800-01-01 }.weekday() == Weekday::Wednesday);
    check!(date! { 1900-01-01 }.weekday() == Weekday::Monday);
    check!(date! { 2000-01-01 }.weekday() == Weekday::Saturday);
    check!(date! { 2100-01-01 }.weekday() == Weekday::Friday);
    check!(date! { 2200-01-01 }.weekday() == Weekday::Wednesday);
    check!(date! { 2300-01-01 }.weekday() == Weekday::Monday);
    check!(date! { 2400-01-01 }.weekday() == Weekday::Saturday);
    check!(date! { 2500-01-01 }.weekday() == Weekday::Friday);
  }

  #[test]
  fn test_display() {
    for (weekday, weekday_str, weekday_abbv_str) in [
      (Weekday::Sunday, "Sunday", "Sun"),
      (Weekday::Monday, "Monday", "Mon"),
      (Weekday::Tuesday, "Tuesday", "Tue"),
      (Weekday::Wednesday, "Wednesday", "Wed"),
      (Weekday::Thursday, "Thursday", "Thu"),
      (Weekday::Friday, "Friday", "Fri"),
      (Weekday::Saturday, "Saturday", "Sat"),
    ] {
      check!(weekday.to_string() == weekday_str);
      check!(weekday.abbv() == weekday_abbv_str);
    }
  }
}

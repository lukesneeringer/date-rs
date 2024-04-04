use std::fmt::Display;

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
  use crate::interval::DateInterval;

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
      date += DateInterval::new(1);
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

    // And some around now...
    check!(date! { 2021-07-04 }.weekday() == Weekday::Sunday);
    check!(date! { 2024-03-31 }.weekday() == Weekday::Sunday);
    check!(date! { 2024-11-28 }.weekday() == Weekday::Thursday);
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

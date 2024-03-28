//! The `date-rs` crate provides a simple, easy-to-use `Date` struct (and corresponding macro).
//! Date provides storage for a single Gregorian calendar date.
//!
//! `Date` can currently store any valid calendar date between years -65,536 and -65,535, although
//! this may change in the future if its internal representation changes.
//!
//! ## Examples
//!
//! Making a date:
//!
//! ```rs
//! use date::Date;
//!
//! let date = Date::new(2012, 4, 21);
//! ```
//!
//! You can also use the `date!` macro to get a syntax resembling a date literal:
//!
//! ```rs
//! use date::date;
//!
//! let date = date! { 2012-04-21 };
//! ```
//!
//! ## Features
//!
//! `date-rs` ships with the following features:
//!
//! - **`diesel-pg`**: Enables interop with PostgreSQL `DATE` columns using Diesel.
//! - **`serde`**: Enables serialization and desearialization with `serde`. _(Enabled by default.)_

use std::fmt;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Construct a date from a `YYYY-MM-DD` literal.
///
/// ## Examples
///
/// ```
/// # use date::date;
/// let d = date! { 2024-01-01 };
/// assert_eq!(d.year(), 2024);
/// assert_eq!(d.month(), 1);
/// assert_eq!(d.day(), 1);
/// ```
#[macro_export]
macro_rules! date {
  ($y:literal-$m:literal-$d:literal) => {{
    #[allow(clippy::zero_prefixed_literal)]
    {
      $crate::Date::new($y, $m, $d)
    }
  }};
}

#[cfg(feature = "diesel-pg")]
mod db;
mod format;
mod interval;
mod parse;
#[cfg(feature = "serde")]
mod serde;
mod utils;
mod weekday;

pub use interval::DateInterval;
use utils::days_in_year;
pub use weekday::Weekday;

/// A representation of a single date.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "diesel-pg", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel-pg", diesel(sql_type = ::diesel::sql_types::Date))]
pub struct Date {
  pub(crate) year: i16,
  pub(crate) day_of_year_0: u16,
}

impl Date {
  /// Construct a new `Date` from the provided year, month, and day.
  ///
  /// ## Examples
  ///
  /// ```
  /// use date::Date;
  /// let date = Date::new(2012, 4, 21);
  /// ```
  ///
  /// ## Panic
  ///
  /// This function panics if it receives "out-of-bounds" values (e.g. "March 32" or "February
  /// 30"). However, it can be convenient to be able to send such values to avoid having to handle
  /// overflow yourself; use [`Date::overflowing_new`] for this purpose.
  pub const fn new(year: i16, month: u8, day: u8) -> Self {
    const MONTH_DAYS: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    assert!(month >= 1 && month <= 12, "Month out-of-bounds");
    assert!(day >= 1 && day <= MONTH_DAYS[month as usize - 1], "Day out-of-bounds");
    if month == 2 && day == 29 {
      assert!(utils::is_leap_year(year), "February 29 only occurs on leap years")
    }

    // Get the proper day of the year.
    let day_of_year_0 = utils::bounds(year)[(month - 1) as usize] + day as u16 - 1;
    Self { year, day_of_year_0 }
  }

  /// Construct a new `Date` based on the Unix timestamp.
  ///
  /// ## Examples
  ///
  /// ```
  /// use date::date;
  /// use date::Date;
  ///
  /// let day_one = Date::from_timestamp(0);
  /// assert_eq!(day_one, date! { 1970-01-01 });
  /// let later = Date::from_timestamp(15_451 * 86_400);
  /// assert_eq!(later, date! { 2012-04-21 });
  /// ```
  ///
  /// Negative timestamps are also supported:
  ///
  /// ```
  /// # use date::date;
  /// # use date::Date;
  /// let before_unix_era = Date::from_timestamp(-1);
  /// assert_eq!(before_unix_era, date! { 1969-12-31 });
  /// let hobbit_publication = Date::from_timestamp(-11_790 * 86_400);
  /// assert_eq!(hobbit_publication, date! { 1937-09-21 });
  /// ```
  pub const fn from_timestamp(unix_timestamp: i64) -> Self {
    let mut year = 1970;
    let mut day_count: i32 = unix_timestamp.div_euclid(86_400) as i32;

    // Is our date prior to 1970; if so, decrement the year. Intentionally overshoot; the remaining
    // day count will effectively be the month and day.
    while day_count < 0 {
      year -= 1;
      day_count += days_in_year(year) as i32;
    }

    // Work our way up through the years until we don't have enough days left.
    while day_count > days_in_year(year) as i32 {
      day_count -= days_in_year(year) as i32;
      year += 1;
    }

    Self { year, day_of_year_0: day_count as u16 }
  }

  /// Construct a new `Date` from the provided year, month, and day.
  ///
  /// This function accepts "overflow" values that would lead to invalid dates, and canonicalizes
  /// them to correct dates, allowing for some math to be done on the inputs without needing to
  /// perform overflow checks yourself.
  ///
  /// For example, it's legal to send "March 32" to this function, and it will yield April 1 of the
  /// same year. It's also legal to send a `month` or `day` value of zero, and it will conform to
  /// the month or day (respectively) prior to the first.
  pub const fn overflowing_new(year: i16, month: u8, day: u8) -> Self {
    let mut year = year;
    let mut month = month;
    let mut day = day;

    // Handle month overflows.
    while month > 12 {
      year += 1;
      month -= 12;
    }
    if month == 0 {
      year -= 1;
      month = 12;
    }
    if month == 1 && day == 0 {
      year -= 1;
      month = 12;
      day = 31;
    }

    // Get the proper day of the year.
    let mut day_of_year_0 = utils::bounds(year)[(month - 1) as usize] + day as u16 - 1;

    // Handle day overflows.
    while day_of_year_0 >= utils::days_in_year(year) {
      day_of_year_0 -= utils::days_in_year(year);
      year += 1;
    }

    // Return the date.
    Self { year, day_of_year_0 }
  }

  /// Return true if this date is during a leap year, false otherwise.
  pub(crate) const fn is_leap_year(&self) -> bool {
    utils::is_leap_year(self.year())
  }
}

impl Date {
  /// Returns the year number in the calendar date.
  #[inline]
  pub const fn year(&self) -> i16 {
    self.year
  }

  /// Returns the month number, starting from 1.
  ///
  /// The return value ranges from 1 to 12.
  #[inline]
  pub const fn month(&self) -> u8 {
    macro_rules! month {
      ($($m:literal),*) => {{
        let bounds = utils::bounds(self.year);
        $(if bounds[$m] > self.day_of_year_0 {
          $m
        })else*
        else { 12 }
      }}
    }
    month!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
  }

  /// Returns the day of the month, starting from 1.
  ///
  /// The return value ranges from 1 to 31. (The last day of the month differs by months.)
  #[inline]
  pub const fn day(&self) -> u8 {
    macro_rules! day {
      ($($m:literal),*) => {{
        let bounds = utils::bounds(self.year);
        ($(if bounds[$m] <= self.day_of_year_0 {
          self.day_of_year_0 - bounds[$m] + 1
        })else*
        else { self.day_of_year_0 + 1 }) as u8
      }}
    }
    day!(11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)
  }

  /// Return the weekday corresponding to the given date.
  #[inline]
  pub const fn weekday(&self) -> Weekday {
    // Implementation for this explained at [Art of Memory][aom].
    //
    // [aom]: https://artofmemory.com/blog/how-to-calculate-the-day-of-the-week/
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

impl Date {
  /// The Unix timestamp for this date at midnight UTC.
  ///
  /// ## Examples
  ///
  /// ```
  /// # use date::date;
  /// assert_eq!(date! { 1969-12-31 }.timestamp(), -86_400);
  /// assert_eq!(date! { 1970-01-01 }.timestamp(), 0);
  /// assert_eq!(date! { 1970-01-05 }.timestamp(), 4 * 86_400);
  /// assert_eq!(date! { 2012-04-21 }.timestamp(), 1334966400);
  /// ```
  pub const fn timestamp(&self) -> i64 {
    let mut day_count = 0;

    // Add the days for all completed years.
    //
    // If the year is prior to 1970, also add (subtract, really) the days for the year under
    // consideration; that way we have a consistent baseline and can always add the month/day
    // values.
    match self.year >= 1970 {
      true => {
        let mut cursor = 1970;
        while cursor < self.year {
          day_count += days_in_year(cursor) as i64;
          cursor += 1;
        }
      },
      false => {
        let mut cursor = 1969;
        while cursor >= self.year {
          day_count -= days_in_year(cursor) as i64;
          cursor -= 1;
        }
      },
    }

    // Add the month and day.
    (day_count + self.day_of_year_0 as i64) * 86_400
  }
}

impl Date {
  /// The date representing today, according to the system local clock.
  ///
  /// ## Panic
  ///
  /// This function will panic if the system clock is set to a time prior to January 1, 1970, or if
  /// the local time zone can not be determined.
  #[cfg(feature = "tzdb")]
  pub fn today() -> Self {
    let tz = tzdb::local_tz().expect("Could not determine local time zone");
    let now =
      now().duration_since(UNIX_EPOCH).expect("system time set prior to 1970").as_secs() as i64;
    let offset = tz
      .find_local_time_type(now)
      .expect("Local time zone lacks information for this timestamp")
      .ut_offset() as i64;
    Self::from_timestamp(now + offset)
  }

  /// The date representing today, in the provided time zone.
  #[cfg(feature = "tzdb")]
  pub fn today_tz(tz: &'static str) -> anyhow::Result<Self> {
    let tz = tzdb::tz_by_name(tz).ok_or(anyhow::format_err!("Time zone not found: {}", tz))?;
    let now = now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let offset = tz.find_local_time_type(now)?.ut_offset() as i64;
    Ok(Self::from_timestamp(now + offset))
  }

  /// The date representing today, in UTC.
  ///
  /// ## Panic
  ///
  /// This function will panic if the system clock is set to a time prior to January 1, 1970.
  pub fn today_utc() -> Self {
    let now = now().duration_since(UNIX_EPOCH).expect("system time set prior to 1970").as_secs();
    Self::from_timestamp(now as i64)
  }
}

impl Date {
  /// Format the date according to the provided `strftime` specifier.
  #[doc = include_str!("../support/date-format.md")]
  ///
  #[doc = include_str!("../support/padding.md")]
  ///
  #[doc = include_str!("../support/plain-characters.md")]
  pub fn format<'a>(&'a self, format_str: &'a str) -> format::FormattedDate {
    format::FormattedDate { date: self, format: format_str }
  }
}

impl fmt::Debug for Date {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.format("%Y-%m-%d"))
  }
}

impl fmt::Display for Date {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.format("%Y-%m-%d"))
  }
}

#[cfg(not(test))]
fn now() -> SystemTime {
  SystemTime::now()
}

#[cfg(test)]
use tests::now;

#[cfg(test)]
mod tests {
  use std::cell::RefCell;

  use assert2::check;

  use super::*;

  thread_local! {
    static MOCK_TIME: RefCell<Option<SystemTime>> = const { RefCell::new(None) };
  }

  fn set_now(time: SystemTime) {
    MOCK_TIME.with(|cell| *cell.borrow_mut() = Some(time));
  }

  fn clear_now() {
    MOCK_TIME.with(|cell| *cell.borrow_mut() = None);
  }

  pub(super) fn now() -> SystemTime {
    MOCK_TIME.with(|cell| cell.borrow().as_ref().cloned().unwrap_or_else(SystemTime::now))
  }

  #[test]
  fn test_ymd_readback() {
    for year in [2020, 2022, 2100] {
      for month in 1..=12 {
        let days = match month {
          1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
          4 | 6 | 9 | 11 => 30,
          2 => match utils::is_leap_year(year) {
            true => 29,
            false => 28,
          },
          #[cfg(not(tarpaulin_include))]
          _ => panic!("Unreachable"),
        };
        for day in 1..=days {
          let date = Date::new(year, month, day);
          check!(date.year() == year);
          check!(date.month() == month);
          check!(date.day() == day);
        }
      }
    }
  }

  #[test]
  #[should_panic]
  fn test_overflow_panic_day() {
    Date::new(2012, 4, 31);
  }

  #[test]
  #[should_panic]
  fn test_overflow_panic_month() {
    Date::new(2012, 13, 1);
  }

  #[test]
  #[should_panic]
  fn test_overflow_panic_ly() {
    Date::new(2100, 2, 29);
  }

  #[test]
  #[allow(clippy::zero_prefixed_literal)]
  fn test_ymd_overflow() {
    macro_rules! overflows_to {
      ($y1:literal-$m1:literal-$d1:literal
          == $y2:literal-$m2:literal-$d2:literal) => {
        let date1 = Date::overflowing_new($y1, $m1, $d1);
        let date2 = Date::new($y2, $m2, $d2);
        check!(date1 == date2);
      };
    }
    overflows_to! { 2022-01-32 == 2022-02-01 };
    overflows_to! { 2022-02-29 == 2022-03-01 };
    overflows_to! { 2022-03-32 == 2022-04-01 };
    overflows_to! { 2022-04-31 == 2022-05-01 };
    overflows_to! { 2022-05-32 == 2022-06-01 };
    overflows_to! { 2022-06-31 == 2022-07-01 };
    overflows_to! { 2022-07-32 == 2022-08-01 };
    overflows_to! { 2022-08-32 == 2022-09-01 };
    overflows_to! { 2022-09-31 == 2022-10-01 };
    overflows_to! { 2022-10-32 == 2022-11-01 };
    overflows_to! { 2022-11-31 == 2022-12-01 };
    overflows_to! { 2022-12-32 == 2023-01-01 };
    overflows_to! { 2022-01-00 == 2021-12-31 };
    overflows_to! { 2022-02-00 == 2022-01-31 };
    overflows_to! { 2022-03-00 == 2022-02-28 };
    overflows_to! { 2022-04-00 == 2022-03-31 };
    overflows_to! { 2022-05-00 == 2022-04-30 };
    overflows_to! { 2022-06-00 == 2022-05-31 };
    overflows_to! { 2022-07-00 == 2022-06-30 };
    overflows_to! { 2022-08-00 == 2022-07-31 };
    overflows_to! { 2022-09-00 == 2022-08-31 };
    overflows_to! { 2022-10-00 == 2022-09-30 };
    overflows_to! { 2022-11-00 == 2022-10-31 };
    overflows_to! { 2022-12-00 == 2022-11-30 };
    overflows_to! { 2020-02-30 == 2020-03-01 };
    overflows_to! { 2020-03-00 == 2020-02-29 };
    overflows_to! { 2022-01-45 == 2022-02-14 };
    overflows_to! { 2022-13-15 == 2023-01-15 };
    overflows_to! { 2022-00-15 == 2021-12-15 };
  }

  #[test]
  fn test_display() {
    check!(date! { 2012-04-21 }.to_string() == "2012-04-21");
    check!(format!("{:?}", date! { 2012-04-21 }) == "2012-04-21");
  }

  #[test]
  fn test_today() {
    set_now(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400));
    check!(Date::today_utc() == date! { 1970-01-02 });
    clear_now();
  }

  #[cfg(feature = "tzdb")]
  #[test]
  fn test_today_tz() -> Result<()> {
    set_now(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400));
    check!([date! { 1970-01-01 }, date! { 1970-01-02 }].contains(&Date::today()));
    check!(Date::today_tz("America/New_York")? == date! { 1970-01-01 });
    clear_now();
    Ok(())
  }
}

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

use std::fmt;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use strptime::ParseError;
use strptime::ParseResult;
use strptime::Parser;

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
pub(crate) mod interval; // FIXME: Change to `pub` in 1.0.
pub mod iter;
#[cfg(feature = "serde")]
mod serde;
mod utils;
mod weekday;

pub use interval::DateInterval; // FIXME: Remove in 1.0.
pub use interval::MonthInterval; // FIXME: Remove in 1.0.
pub use weekday::Weekday;

/// A representation of a single date.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "diesel-pg", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel-pg", diesel(sql_type = ::diesel::sql_types::Date))]
#[repr(transparent)]
pub struct Date(i32);

impl Date {
  /// Construct a new `Date` from the provided year, month, and day.
  ///
  /// ## Examples
  ///
  /// ```
  /// use date::Date;
  /// let date = Date::new(2012, 4, 21);
  /// assert_eq!(date.year(), 2012);
  /// assert_eq!(date.month(), 4);
  /// assert_eq!(date.day(), 21);
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

    // The algorithm to convert from a civil year/month/day to the number of days that have elapsed
    // since the epoch is taken from here:
    // https://howardhinnant.github.io/date_algorithms.html#days_from_civil
    let year = year as i32 - if month <= 2 { 1 } else { 0 };
    let month = month as i32;
    let day = day as i32;
    let era: i32 = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Self(era * 146097 + day_of_era - 719468)
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
    let day_count = unix_timestamp.div_euclid(86_400) as i32;
    Self(day_count)
  }

  // FIXME: Make `tz` take a `TimeZoneRef<'static>` in 1.0
  // ---

  /// The date on which the given timestamp occurred in the provided time zone.
  #[cfg(feature = "tzdb")]
  pub fn from_timestamp_tz(unix_timestamp: i64, tz: &'static str) -> anyhow::Result<Self> {
    let tz = tzdb::tz_by_name(tz).ok_or(anyhow::format_err!("Time zone not found: {}", tz))?;
    let offset = tz.find_local_time_type(unix_timestamp)?.ut_offset() as i64;
    Ok(Self::from_timestamp(unix_timestamp + offset))
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
    if day == 0 {
      if month <= 1 {
        year -= 1;
        month += 11;
      } else {
        month -= 1;
      }
      day = utils::days_in_month(year, month);
    }
    if month == 0 {
      year -= 1;
      month = 12;
    }
    while day > utils::days_in_month(year, month) {
      day -= utils::days_in_month(year, month);
      month += 1;
      if month == 13 {
        year += 1;
        month = 1;
      }
    }

    // Return the date.
    Self::new(year, month, day)
  }

  /// Parse a date from a string, according to the provided format string.
  pub fn parse(date_str: impl AsRef<str>, date_fmt: &'static str) -> ParseResult<Date> {
    let parser = Parser::new(date_fmt);
    let raw_date = parser.parse(date_str)?.date()?;
    Ok(raw_date.into())
  }
}

impl Date {
  /// The year, month, and day for the given date.
  pub(crate) const fn ymd(&self) -> (i16, u8, u8) {
    // The algorithm to convert from a civil year/month/day to the number of days that have elapsed
    // since the epoch is taken from here:
    // https://howardhinnant.github.io/date_algorithms.html#civil_from_days
    let shifted = self.0 + 719468; // Days from March 1, 0 A.D.
    let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
    let doe = shifted - era * 146_097; // day of era: [0, 146_097)
    let year_of_era = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = doe - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (year as i16 + if month <= 2 { 1 } else { 0 }, month as u8, day as u8)
  }

  /// Returns the year number in the calendar date.
  #[inline]
  pub const fn year(&self) -> i16 {
    self.ymd().0
  }

  /// Returns the month number, starting from 1.
  ///
  /// The return value ranges from 1 to 12.
  #[inline]
  pub const fn month(&self) -> u8 {
    self.ymd().1
  }

  /// Returns the day of the month, starting from 1.
  ///
  /// The return value ranges from 1 to 31. (The last day of the month differs by months.)
  #[inline]
  pub const fn day(&self) -> u8 {
    self.ymd().2
  }

  /// The day of the current year. Range: `[1, 366]`
  #[inline]
  pub const fn day_of_year(&self) -> u16 {
    (self.0 - Date::new(self.year() - 1, 12, 31).0) as u16
  }

  /// The week number of the year (between 0 and 53, inclusive), with a new week starting each
  /// Sunday.
  ///
  /// Week 1 begins on the first Sunday of the year; leading days before that are part of week 0.
  pub const fn week(&self) -> u16 {
    let jan1 = Date::new(self.year(), 1, 1);
    let first_sunday = jan1.0 + if self.0 % 7 == 3 { 0 } else { 7 } - (self.0 + 4) % 7;
    ((self.0 - first_sunday).div_euclid(7) + 1) as u16
  }

  /// Return the weekday corresponding to the given date.
  #[inline]
  pub const fn weekday(&self) -> Weekday {
    match (self.0 + 4) % 7 {
      0 => Weekday::Sunday,
      1 | -6 => Weekday::Monday,
      2 | -5 => Weekday::Tuesday,
      3 | -4 => Weekday::Wednesday,
      4 | -3 => Weekday::Thursday,
      5 | -2 => Weekday::Friday,
      6 | -1 => Weekday::Saturday,
      #[cfg(not(tarpaulin_include))]
      _ => panic!("Unreachable: Anything % 7 must be within -6 to 6"),
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
    self.0 as i64 * 86_400
  }

  /// The Unix timestamp for this date at midnight in the given time zone.
  #[cfg(feature = "tzdb")]
  pub fn timestamp_tz(&self, tz: &'static str) -> anyhow::Result<i64> {
    let tz = tzdb::tz_by_name(tz).ok_or(anyhow::format_err!("Time zone not found: {}", tz))?;
    let offset = tz.find_local_time_type(self.timestamp())?.ut_offset() as i64;
    Ok(self.timestamp() - offset)
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

impl Date {
  /// An iterator of dates beginning with this date, and ending with the provided end date
  /// (inclusive).
  pub fn iter_through(&self, end: Date) -> iter::DateIterator {
    iter::DateIterator::new(self, end)
  }
}

impl Date {
  /// The maximum date that can be represented.
  pub const MAX: Self = Date::new(32767, 12, 31);
  /// The minimum date that can be represented.
  pub const MIN: Self = Date::new(-32768, 1, 1);
}

#[cfg(feature = "easter")]
impl Date {
  /// The date of Easter in the Gregorian calendar for the given year.
  pub const fn easter(year: i16) -> Self {
    assert!(year >= 1583 || year <= 9999, "Year out of bounds");
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let j = c % 4;
    let k = (32 + 2 * e + 2 * i - h - j) % 7;
    let l = (a + 11 * h + 22 * k) / 451;
    let month = (h + k - 7 * l + 114) / 31;
    let day = (h + k - 7 * l + 114) % 31 + 1;
    Self::new(year, month as u8, day as u8)
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

impl FromStr for Date {
  type Err = ParseError;

  fn from_str(s: &str) -> ParseResult<Self> {
    Self::parse(s, "%Y-%m-%d")
  }
}

impl From<strptime::RawDate> for Date {
  fn from(value: strptime::RawDate) -> Self {
    Self::new(value.year(), value.month(), value.day())
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
  fn test_internal_repr() {
    check!(date! { 1969-12-31 }.0 == -1);
    check!(date! { 1970-01-01 }.0 == 0);
    check!(date! { 1970-01-02 }.0 == 1);
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
    overflows_to! { 2022-00-00 == 2021-11-30 };
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
  fn test_week() {
    check!(date! { 2022-01-01 }.week() == 0); // Saturday
    check!(date! { 2022-01-02 }.week() == 1); // Sunday
    check!(date! { 2023-01-01 }.week() == 1); // Sunday
    check!(date! { 2023-12-31 }.week() == 53); // Sunday
    check!(date! { 2024-01-01 }.week() == 0); // Monday
    check!(date! { 2024-01-07 }.week() == 1); // Sunday
    check!(date! { 2024-01-08 }.week() == 1); // Monday
    check!(date! { 2024-01-14 }.week() == 2); // Sunday
  }

  #[test]
  fn test_today() {
    set_now(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400));
    check!(Date::today_utc() == date! { 1970-01-02 });
    clear_now();
  }

  #[cfg(feature = "tzdb")]
  #[test]
  fn test_today_tz() -> anyhow::Result<()> {
    set_now(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400));
    check!([date! { 1970-01-01 }, date! { 1970-01-02 }].contains(&Date::today()));
    check!(Date::today_tz("America/New_York")? == date! { 1970-01-01 });
    clear_now();
    Ok(())
  }

  #[cfg(feature = "tzdb")]
  #[test]
  fn test_timestamp_tz() -> anyhow::Result<()> {
    check!(Date::from_timestamp_tz(1335020400, "America/New_York")? == date! { 2012-04-21 });
    check!(Date::from_timestamp_tz(0, "America/Los_Angeles")? == date! { 1969-12-31 });
    check!(date! { 2012-04-21 }.timestamp_tz("America/New_York")? == 1334980800);
    Ok(())
  }

  #[cfg(feature = "easter")]
  #[test]
  fn test_easter() {
    check!(Date::easter(2013) == date! { 2013-03-31 });
    check!(Date::easter(2014) == date! { 2014-04-20 });
    check!(Date::easter(2015) == date! { 2015-04-05 });
    check!(Date::easter(2016) == date! { 2016-03-27 });
    check!(Date::easter(2017) == date! { 2017-04-16 });
    check!(Date::easter(2018) == date! { 2018-04-01 });
    check!(Date::easter(2019) == date! { 2019-04-21 });
    check!(Date::easter(2020) == date! { 2020-04-12 });
    check!(Date::easter(2021) == date! { 2021-04-04 });
    check!(Date::easter(2022) == date! { 2022-04-17 });
    check!(Date::easter(2023) == date! { 2023-04-09 });
    check!(Date::easter(2024) == date! { 2024-03-31 });
    check!(Date::easter(2025) == date! { 2025-04-20 });
    check!(Date::easter(2026) == date! { 2026-04-05 });
    check!(Date::easter(2027) == date! { 2027-03-28 });
    check!(Date::easter(2028) == date! { 2028-04-16 });
    check!(Date::easter(2029) == date! { 2029-04-01 });
    check!(Date::easter(2030) == date! { 2030-04-21 });
    check!(Date::easter(2031) == date! { 2031-04-13 });
    check!(Date::easter(2032) == date! { 2032-03-28 });
    check!(Date::easter(2033) == date! { 2033-04-17 });
    check!(Date::easter(2034) == date! { 2034-04-09 });
    check!(Date::easter(2035) == date! { 2035-03-25 });
  }

  #[test]
  fn test_from_str() -> ParseResult<()> {
    check!("2012-04-21".parse::<Date>()? == date! { 2012-04-21 });
    check!("2012-4-21".parse::<Date>().is_err());
    check!("04/21/2012".parse::<Date>().is_err());
    check!("12-04-21".parse::<Date>().is_err());
    check!("foo".parse::<Date>().map_err(|e| e.to_string()).unwrap_err().contains("foo"));
    Ok(())
  }

  #[test]
  fn test_parse() -> ParseResult<()> {
    check!(Date::parse("04/21/12", "%m/%d/%y")? == date! { 2012-04-21 });
    check!(Date::parse("Saturday, April 21, 2012", "%A, %B %-d, %Y")? == date! { 2012-04-21 });
    Ok(())
  }
}

//! Intervals that can be added to or subtracted from dates.
//!
//! In addition, dates can be subtracted from one another, and the result is a [`DateInterval`].

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use crate::Date;
use crate::utils;

/// An interval of days.
///
/// Intervals can be positive or negative, in part because the difference between two dates is
/// expressed as a [`DateInterval`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DateInterval {
  days: i32,
}

impl DateInterval {
  /// A reprensentation of a given number of days.
  #[inline]
  pub const fn new(days: i32) -> Self {
    Self { days }
  }

  /// The number of days this interval represents.
  pub const fn days(&self) -> i32 {
    self.days
  }

  /// The absolute value of this interval.
  pub const fn abs(self) -> Self {
    Self { days: self.days.abs() }
  }
}

impl Neg for DateInterval {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self { days: -self.days }
  }
}

impl Add<DateInterval> for Date {
  type Output = Date;

  /// Return a new `Date` that is the given number of days later.
  fn add(self, interval: DateInterval) -> Self::Output {
    Date(self.0 + interval.days())
  }
}

impl AddAssign<DateInterval> for Date {
  fn add_assign(&mut self, interval: DateInterval) {
    self.0 += interval.days();
  }
}

impl Sub<DateInterval> for Date {
  type Output = Date;

  /// Return a new `Date` that is the given number of days earlier.
  fn sub(self, interval: DateInterval) -> Self::Output {
    Date(self.0 - interval.days())
  }
}

impl SubAssign<DateInterval> for Date {
  fn sub_assign(&mut self, interval: DateInterval) {
    self.0 -= interval.days();
  }
}

impl Sub<Date> for Date {
  type Output = DateInterval;

  fn sub(self, rhs: Date) -> Self::Output {
    DateInterval::new(self.0 - rhs.0)
  }
}

/// An interval of months.
///
/// Unlike [`DateInterval`], this only represents positive numbers of months, because we never
/// receive this object as a result of subtracting one [`Date`] from another; instead, this
/// object's sole purpose is to create month intervals to add or subtract from dates.
///
/// In the event that a month interval is added to a date where the day of the month exceeds the
/// number of days in the result month, the day is set to the final day of the result month.
/// Therefore, adding one month to `2021-01-31` will return `2021-02-28`.
///
/// Importantly, this means that addition and subtraction are not necessarily communitive.
///
/// ## Example
///
/// ```
/// use date::date;
/// use date::interval::MonthInterval;
///
/// assert_eq!(date! { 2012-04-21 } + MonthInterval::new(3), date! { 2012-07-21 });
/// assert_eq!(date! { 2021-12-31 } + MonthInterval::new(2), date! { 2022-02-28 });
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct MonthInterval {
  months: u8,
}

impl MonthInterval {
  /// Create a new month interval
  pub const fn new(months: u8) -> Self {
    assert!(months <= 255 - 12, "MonthInterval out of bounds.");
    Self { months }
  }

  /// The number of months this interval represents.
  pub const fn months(&self) -> u8 {
    self.months
  }
}

impl Add<MonthInterval> for Date {
  type Output = Self;

  fn add(self, interval: MonthInterval) -> Self {
    saturated_date(self.year(), self.month() + interval.months(), self.day())
  }
}

impl Sub<MonthInterval> for Date {
  type Output = Self;

  fn sub(self, interval: MonthInterval) -> Self {
    let year = self.year() - interval.months().div_ceil(12) as i16;
    saturated_date(year, self.month() + (12 - interval.months() % 12), self.day())
  }
}

/// If the provided day falls after the final day of the month, return the final day of the month.
fn saturated_date(year: i16, month: u8, day: u8) -> Date {
  Date::overflowing_new(year, month, match month % 12 {
    1 | 3 | 5 | 7 | 8 | 10 | 0 => day.min(31),
    4 | 6 | 9 | 11 => day.min(30),
    2 => day.min(if utils::is_leap_year(year + month as i16 / 12) { 29 } else { 28 }),
    _ => unreachable!("n % 12 is always 0..=11"),
  })
}

#[cfg(test)]
#[allow(clippy::zero_prefixed_literal)]
mod tests {
  use assert2::check;

  use super::*;

  #[test]
  fn test_add_sub() {
    macro_rules! prove {
      ($y1:literal-$m1:literal-$d1:literal + $dur:literal
          == $y2:literal-$m2:literal-$d2:literal) => {
        // Check `+`.
        check!(Date::new($y1, $m1, $d1) + DateInterval::new($dur) == Date::new($y2, $m2, $d2));

        // Check `+=`.
        let mut date = Date::new($y1, $m1, $d1);
        date += DateInterval::new($dur);
        check!(date == Date::new($y2, $m2, $d2));
      };
      ($y1:literal-$m1:literal-$d1:literal - $dur:literal
          == $y2:literal-$m2:literal-$d2:literal) => {
        // Check `-`.
        check!(Date::new($y1, $m1, $d1) - DateInterval::new($dur) == Date::new($y2, $m2, $d2));

        // Check `-=`.
        let mut date = Date::new($y1, $m1, $d1);
        date -= DateInterval::new($dur);
        check!(date == Date::new($y2, $m2, $d2));
      };
    }

    // Movement by a day.
    prove! { 2019-12-31 + 1 == 2020-01-01 };
    prove! { 2020-12-31 + 1 == 2021-01-01 };
    prove! { 2020-01-01 - 1 == 2019-12-31 };
    prove! { 2021-01-01 - 1 == 2020-12-31 };
    prove! { 2019-06-30 + 1 == 2019-07-01 };
    prove! { 2020-07-01 - 1 == 2020-06-30 };
    prove! { 2020-06-15 + 1 == 2020-06-16 };
    prove! { 2020-06-15 - 1 == 2020-06-14 };

    // Movement by a month (or so).
    prove! {2019-02-15 + 28 == 2019-03-15};
    prove! {2020-02-15 + 29 == 2020-03-15};
    prove! {2019-03-15 - 28 == 2019-02-15};
    prove! {2020-03-15 - 29 == 2020-02-15};

    // Movement by a year.
    prove! {2019-06-30 + 366 == 2020-06-30};
    prove! {2019-06-30 - 365 == 2018-06-30};

    // Movement by multiple years.
    prove! {2019-06-30 + 730 == 2021-06-29};
    prove! {2019-06-30 - 730 == 2017-06-30};
    prove! {2020-06-30 - 366 == 2019-06-30};
    prove! {2015-06-30 + 2555 == 2022-06-28}; // 2555 == 365 * 7
    prove! {2022-06-30 - 2555 == 2015-07-02}; // 2555 == 365 * 7
  }

  #[test]
  fn test_sub_dates() {
    check!(date! { 2012-04-21 } - date! { 2012-04-21 } == DateInterval::new(0));
    check!(date! { 2012-04-22 } - date! { 2012-04-21 } == DateInterval::new(1));
    check!(date! { 2012-04-24 } - date! { 2012-04-21 } == DateInterval::new(3));
    check!(date! { 2012-04-20 } - date! { 2012-04-21 } == DateInterval::new(-1));
    check!(date! { 2012-04-14 } - date! { 2012-04-21 } == DateInterval::new(-7));
    check!(date! { 2012-01-02 } - date! { 2011-12-30 } == DateInterval::new(3));
    check!(date! { 2011-12-30 } - date! { 2012-01-02 } == DateInterval::new(-3));
    check!(date! { 2018-06-01 } - date! { 2016-06-01 } == DateInterval::new(730));

    // Identity
    check!(
      date! { 2012-04-18 } + (date! { 2012-04-21 } - date! { 2012-04-18 }) == date! { 2012-04-21 }
    );
  }

  #[test]
  fn test_add_sub_months() {
    macro_rules! prove {
      ($y1:literal-$m1:literal-$d1:literal + $dur:literal months
          == $y2:literal-$m2:literal-$d2:literal) => {
        // Check `+`.
        check!(Date::new($y1, $m1, $d1) + MonthInterval::new($dur) == Date::new($y2, $m2, $d2));

        // Check `-`.
        check!(Date::new($y2, $m2, $d2) - MonthInterval::new($dur) == Date::new($y1, $m1, $d1));
      };
    }

    prove! { 2020-04-15 + 3 months == 2020-07-15 };
    prove! { 2019-11-30 + 5 months == 2020-04-30 };
    prove! { 2023-12-15 + 1 months == 2024-01-15 };
    prove! { 2012-04-21 + 18 months == 2013-10-21 };
    prove! { 2023-11-28 + 18 months == 2025-05-28 };

    // Coercsion of days (non-communicative).
    check!(date! { 2020-01-31 } + MonthInterval::new(1) == date! { 2020-02-29 });
  }
}

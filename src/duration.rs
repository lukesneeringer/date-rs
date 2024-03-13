use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use crate::utils;
use crate::Date;

/// Duration with day-level precision only.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Duration {
  pub days: i32,
}

impl Duration {
  /// A reprensentation of a given number of days.
  #[inline]
  pub const fn days(days: i32) -> Self {
    Self { days }
  }

  /// The absolute value of this duration.
  const fn abs(self) -> Self {
    Self { days: self.days.abs() }
  }
}

impl Neg for Duration {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self { days: -self.days }
  }
}

impl Add<Duration> for Date {
  type Output = Date;

  /// Return a new `Date` that is the given number of days later.
  fn add(self, duration: Duration) -> Self::Output {
    if duration.days < 0 {
      return self.sub(duration.abs());
    }
    let mut year = self.year;
    let mut day_of_year_0 = self.day_of_year_0 + duration.days as u16;
    while day_of_year_0 >= utils::days_in_year(year) {
      day_of_year_0 -= utils::days_in_year(year);
      year += 1;
    }
    Date { year, day_of_year_0 }
  }
}

impl AddAssign<Duration> for Date {
  fn add_assign(&mut self, duration: Duration) {
    if duration.days < 0 {
      return self.sub_assign(duration.abs());
    }
    self.day_of_year_0 += duration.days as u16;
    while self.day_of_year_0 >= utils::days_in_year(self.year) {
      self.day_of_year_0 -= utils::days_in_year(self.year);
      self.year += 1;
    }
  }
}

impl Sub<Duration> for Date {
  type Output = Date;

  /// Return a new `Date` that is the given number of days earlier.
  fn sub(self, duration: Duration) -> Self::Output {
    if duration.days < 0 {
      return self.add(duration.abs());
    }

    let mut year = self.year();
    let mut subtracand = duration.days as u16;

    // Knock off any full years.
    while subtracand > utils::days_in_year(year) {
      year -= 1;
      subtracand -= utils::days_in_year(year);
    }

    // Is the subtracand smaller? Then it's an earlier date in the same year,
    // and we can just return that.
    //
    // On the other hand, if the subtracand is larger, that means it's a later
    // date in the immediately prior year.
    match subtracand <= self.day_of_year_0 {
      true => Date { year, day_of_year_0: self.day_of_year_0 - subtracand },
      false => Date {
        year: year - 1,
        day_of_year_0: utils::days_in_year(year - 1) + self.day_of_year_0 - subtracand,
      },
    }
  }
}

impl SubAssign<Duration> for Date {
  fn sub_assign(&mut self, duration: Duration) {
    if duration.days < 0 {
      return self.add_assign(duration.abs());
    }
    let mut subtracand = duration.days as u16;

    // Knock off any full years.
    while subtracand > utils::days_in_year(self.year) {
      self.year -= 1;
      subtracand -= utils::days_in_year(self.year);
    }

    // Is the subtracand smaller? Then it's an earlier date in the same year,
    // and we can just set that.
    //
    // On the other hand, if the subtracand is larger, that means it's a later
    // date in the immediately prior year.
    match subtracand <= self.day_of_year_0 {
      true => self.day_of_year_0 -= subtracand,
      false => {
        self.year -= 1;
        self.day_of_year_0 = utils::days_in_year(self.year) + self.day_of_year_0 - subtracand;
      },
    }
  }
}

impl Sub<Date> for Date {
  type Output = Duration;

  fn sub(self, rhs: Date) -> Self::Output {
    if rhs > self {
      return -(rhs - self);
    }
    let year_days: i32 = (rhs.year..=self.year).map(|y| utils::days_in_year(y) as i32).sum();
    Duration::days(
      year_days // 730
        - (utils::days_in_year(self.year) - self.day_of_year_0) as i32 // - 363
        - rhs.day_of_year_0 as i32, // - 363
    )
  }
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
        check!(Date::new($y1, $m1, $d1) + Duration::days($dur) == Date::new($y2, $m2, $d2));

        // Check `+=`.
        let mut date = Date::new($y1, $m1, $d1);
        date += Duration::days($dur);
        check!(date == Date::new($y2, $m2, $d2));
      };
      ($y1:literal-$m1:literal-$d1:literal - $dur:literal
          == $y2:literal-$m2:literal-$d2:literal) => {
        // Check `-`.
        check!(Date::new($y1, $m1, $d1) - Duration::days($dur) == Date::new($y2, $m2, $d2));

        // Check `-=`.
        let mut date = Date::new($y1, $m1, $d1);
        date -= Duration::days($dur);
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
    check!(date! { 2012-04-21 } - date! { 2012-04-21 } == Duration::days(0));
    check!(date! { 2012-04-22 } - date! { 2012-04-21 } == Duration::days(1));
    check!(date! { 2012-04-24 } - date! { 2012-04-21 } == Duration::days(3));
    check!(date! { 2012-04-20 } - date! { 2012-04-21 } == Duration::days(-1));
    check!(date! { 2012-04-14 } - date! { 2012-04-21 } == Duration::days(-7));
    check!(date! { 2012-01-02 } - date! { 2011-12-30 } == Duration::days(3));
    check!(date! { 2011-12-30 } - date! { 2012-01-02 } == Duration::days(-3));
    check!(date! { 2018-06-01 } - date! { 2016-06-01 } == Duration::days(730));

    // Identity
    check!(
      date! { 2012-04-18 } + (date! { 2012-04-21 } - date! { 2012-04-18 }) == date! { 2012-04-21 }
    );
  }
}

//! Iterator over dates

use std::iter::Iterator;

use crate::Date;
use crate::interval::DateInterval;

/// An iterator that will yield dates indefinitely.
pub struct DateIterator {
  cursor: Date,
  end: Date,
}

impl DateIterator {
  pub(crate) const fn new(d: &Date, end: Date) -> Self {
    Self { cursor: *d, end }
  }
}

impl Iterator for DateIterator {
  type Item = Date;

  fn next(&mut self) -> Option<Self::Item> {
    match self.cursor > self.end {
      true => None,
      false => {
        let answer = Some(self.cursor);
        self.cursor += DateInterval::new(1);
        answer
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use assert2::check;

  use super::*;

  #[test]
  fn test_iter() {
    let start = date! { 2012-04-21 };
    check!(start.iter_through(date! { 2012-04-25 }).collect::<Vec<Date>>().len() == 5);
    check!(start.iter_through(date! { 2012-04-21 }).collect::<Vec<Date>>().len() == 1);
    check!(start.iter_through(date! { 2012-04-20 }).collect::<Vec<Date>>().is_empty());
    check!(start.iter_through(Date::MAX).next().unwrap() == date! { 2012-04-21 });
  }
}

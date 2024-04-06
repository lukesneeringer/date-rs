const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const DAYS_IN_MONTH_LY: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Return true if this is a leap year, false otherwise.
pub(crate) const fn is_leap_year(year: i16) -> bool {
  year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Returns the number of days in the month.
pub(crate) const fn days_in_month(year: i16, month: u8) -> u8 {
  (match is_leap_year(year) {
    true => DAYS_IN_MONTH_LY,
    false => DAYS_IN_MONTH,
  })[month as usize - 1]
}

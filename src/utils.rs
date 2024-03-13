/// The last day of each month, as the day of the overall year, indexed from 0
/// (not 1).
///
/// Leap years are one value higher starting at index 3.
const BOUNDS: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const LY_BOUNDS: [u16; 12] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

/// Return true if this is a leap year, false otherwise.
pub(crate) const fn is_leap_year(year: i16) -> bool {
  year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Returns the number of days in the year.
pub(crate) const fn days_in_year(year: i16) -> u16 {
  if is_leap_year(year) { 366 } else { 365 }
}

/// Return bounds adjusted appropriately if this is a leap year.
pub(crate) const fn bounds(year: i16) -> &'static [u16; 12] {
  match is_leap_year(year) {
    true => &LY_BOUNDS,
    false => &BOUNDS,
  }
}

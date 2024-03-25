The following date specifiers are supported:

## Year

| Token | Example | Description                                            |
| ----- | ------- | ------------------------------------------------------ |
| `%C`  | `20`    | Gregorian year divided by 100, zero-padded to 2 digits |
| `%Y`  | `2012`  | Gregorian year, zero-padded to 4 digits                |
| `%y`  | `12`    | Gregorian year modulo 100, zero-padded to 2 digits     |

## Month

| Token        | Example  | Description                                            |
| ------------ | -------- | ------------------------------------------------------ |
| `%B`         | December | Full English name of the month                         |
| `%b` or `%h` | Dec      | Three-letter abbreviation for the month's English name |
| `%m`         | `07`     | Month number (`01`–`12`), zero-padded to 2 digits      |

## Day

| Token | Example    | Description                                                      |
| ----- | ---------- | ---------------------------------------------------------------- |
| `%d`  | `21`       | Day number (`01`–`31`), zero-padded to 2 digits                  |
| `%a`  | `Sat`      | Three-letter abbreviation for the weekday's English name         |
| `%A`  | `Saturday` | Full English name of the weekday                                 |
| `%w`  | `6`        | Integer representing the weekday: Sunday (`0`) to Saturday (`6`) |
| `%u`  | `6`        | Integer representing the weekday: Monday (`1`) to Sunday (`7`)   |
| `%j`  | `112`      | Day of the year (`001`–`366`), zero-padded to 3 digits           |

## Full Date Shortcuts

| Token | Example      | Description                                         |
| ----- | ------------ | --------------------------------------------------- |
| `%D`  | `07/08/01`   | Month-day-year format. Same as %m/%d/%y.            |
| `%F`  | `2001-07-08` | Year-month-day format (ISO 8601). Same as %Y-%m-%d. |
| `%v`  | `8-Jul-2001` | Day-month-year format. Same as %e-%b-%Y.            |

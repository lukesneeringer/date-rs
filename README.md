# Date

[![ci](https://github.com/lukesneeringer/date-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/lukesneeringer/date-rs/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/lukesneeringer/date-rs/branch/main/graph/badge.svg?token=oH2z7PFp06)](https://codecov.io/gh/lukesneeringer/date-rs)
[![release](https://img.shields.io/crates/v/date-rs.svg)](https://crates.io/crates/date-rs)
[![docs](https://img.shields.io/badge/docs-release-blue)](https://docs.rs/date-rs/latest/date/)

The `date` crate provides a simple, easy-to-use `Date` struct (and corresponding macro). Date
provides storage for a single Gregorian calendar date.

`Date` can currently represent any valid calendar date between years -32,768 and 32,767.

## Examples

Making a date:

```rs
use date::Date;

let date = Date::new(2012, 4, 21);
```

You can also use the `date!` macro to get a syntax resembling a date literal:

```rs
use date::date;

let date = date! { 2012-04-21 };
```

## Overflow

`Date` provides an `overflowing_new` function that allows for overflow values (for example,
February 30 or December 32), and maps them accordingly. This allows users to perform certain
mathematical computations without having to do their own overflow checking.

## Features

`date-rs` ships with the following features:

- **`diesel-pg`**: Enables interop with PostgreSQL `DATE` columns using Diesel.
- **`duckdb`**: Enables interop with `duckdb` crate.
- **`easter`**: Enables calculation for the date of Easter (Gregorian calenda).
- **`log`**: Adds `log::kv::ToValue` implementation.
- **`serde`**: Enables serialization and desearialization with `serde`. _(Enabled by default.)_
- **`tz`**: Enables support for time-zone-aware date construction.

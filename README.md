# Date

[![ci](https://github.com/ss151/date-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/ss151/date-rs/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/gh/ss151/date-rs/branch/main/graph/badge.svg?token=FyWXWAuxMH)](https://codecov.io/gh/ss151/date-rs)
[![release](https://api-prd.cloudsmith.io/v1/badges/version/ss151/rust/cargo/date/latest/x/?render=true&show_latest=false&badge_token=gAAAAABle61i4iPL0V1PXCIb3pbBNDLF1sO2N2w4Z68H3otd3wBLKVk-Hk4g1M6ywVqVsKmMMrnWOmFZpdGMkTZ90YKjXMw7yB_hC8vEEHJQUMkQAjDE87M%3D)](https://cloudsmith.io/~ss151/repos/rust/packages/detail/cargo/date/latest/)
[![docs](https://img.shields.io/badge/docs-release-blue)](https://rustdoc.highsignal.tech/date/)

The `date` crate provides a simple, easy-to-use `Date` struct (and corresponding macro). Date
provides storage for a single Gregorian calendar date.

`Date` can currently store any valid calendar date between years -65,536 and -65,535, although this
may change in the future if its internal representation changes.

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
- **`serde`**: Enables serialization and desearialization with `serde`. _(Enabled by default.)_

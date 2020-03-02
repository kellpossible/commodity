# Commodity [![](https://img.shields.io/crates/v/commodity.svg)](https://crates.io/crates/commodity) [![](https://img.shields.io/github/license/kellpossible/commodity)](https://github.com/kellpossible/commodity/blob/master/LICENSE.txt) [![](https://docs.rs/commodity/badge.svg)](https://docs.rs/commodity/)

A library for representing commodities/currencies in Rust, with values backed by the [rust_decimal](https://crates.io/crates/rust_decimal) library.

## Optional Features

The following features can be enabled to provide extra functionality:

+ `serde-support`
  + Enables support for serialization/de-serialization via `serde`

## Example

```rust
use commodity::{Commodity, Currency, CurrencyCode};
use rust_decimal::Decimal;
use std::str::FromStr;

// Create a currency from its iso4317 three character code.
// The currency stores information associated with the currency,
// such as the full name ("United States dollar" for this one).
let usd = Currency::from_alpha3("USD").unwrap();

// Create a commodity with a value of "2.02 USD"
let commodity1 = Commodity::new(Decimal::from_str("2.02").unwrap(), &usd);

// Create commodities using the `from_str` method
let commodity2 = Commodity::from_str("24.00 USD").unwrap();

// Create commodity using a CurrencyCode
let nzd_code = CurrencyCode::from_str("NZD").unwrap();
let commodity3 = Commodity::new(Decimal::from_str("24.00").unwrap(), nzd_code);

// Add two compatible (same currency) commodities, the result has
// the same currency (in this case, "USD").
let commodity4 = commodity1.add(&commodity2).unwrap();

// Try to subtract two incompatible commodities
let result = commodity3.sub(&commodity2);
assert!(result.is_err());
```

//! A library with primatives representing commodities/money.
//!
//! # Optional Features
//!
//! The commodity package has the following optional cargo features:
//!
//! + `serde-support`
//!   + Disabled by default
//!   + Enables support for serialization/de-serialization via `serde`
//!
//! # Usage
//!
//! This library revolves around the [Commodity](Commodity) struct,
//! which stores a value using
//! [rust_decimal::Decimal](rust_decimal::Decimal), and a
//! [CommodityTypeID](CommodityTypeID) which denotes the type of commodity.
//! Commodities with different currencies cannot interact with
//! mathematical operations such as `add`, `sub`, etc, this is checked
//! at runtime.
//!
//! [CommodityTypeID](CommodityTypeID) designed to be used directly when
//! when working with commodities, it is backed by a small fixed size
//! array which supports the [Copy](std::marker::Copy) trait,
//! hopefully making it easier fast, lock-free concurrent code that
//! deals with commodities.
//!
//! [CommodityType](CommodityType) is designed to store useful user-facing
//! information about the currency being referenced by the
//! [CommodityTypeID](CommodityTypeID), such as its full name/description.
//!
//! ```
//! use commodity::{Commodity, CommodityType, CommodityTypeID};
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! // Create a currency from its iso4317 three character code.
//! // The currency stores information associated with the currency,
//! // such as the full name ("United States dollar" for this one).
//! let usd = CommodityType::from_currency_alpha3("USD").unwrap();
//!
//! // Create a commodity with a value of "2.02 USD"
//! let commodity1 = Commodity::new(Decimal::from_str("2.02").unwrap(), &usd);
//!
//! // Create commodities using the `from_str` method
//! let commodity2 = Commodity::from_str("24.00 USD").unwrap();
//!
//! // Create commodity using a CommodityTypeID
//! let nzd_code = CommodityTypeID::from_str("NZD").unwrap();
//! let commodity3 = Commodity::new(Decimal::from_str("24.00").unwrap(), nzd_code);
//!
//! // Add two compatible (same currency) commodities, the result has
//! // the same currency (in this case, "USD").
//! let commodity4 = commodity1.add(&commodity2).unwrap();
//!
//! // Try to subtract two incompatible commodities
//! let result = commodity3.sub(&commodity2);
//! assert!(result.is_err());
//! ```

extern crate arrayvec;
extern crate chrono;
extern crate iso4217;
extern crate rust_decimal;

#[cfg(feature = "serde-support")]
extern crate serde;

#[cfg(test)]
#[cfg(feature = "serde-support")]
extern crate serde_json;

mod commodity;
pub mod exchange_rate;

pub use crate::commodity::*;

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

//! A library with primatives representing commodities/money.
//!
//! # Optional Features
//!
//! The commodity package has the following optional cargo features:
//!
//! + `serde-support`
//!   + Optional
//!   + Enables support for serialization/de-serialization via `serde`
//!
//! # Useage
//!
//! This library revolves around the [Commodity](Commodity) struct,
//! which stores a value using
//! [rust_decimal::Decimal](rust_decimal::Decimal), and a
//! [CurrencyCode](CurrencyCode) which denotes the type of commodity.
//! Commodities with different currencies cannot interact with
//! mathematical operations such as `add`, `sub`, etc, this is checked
//! at runtime.
//!
//! [CurrencyCode](CurrencyCode) designed to be used directly when
//! when working with commodities, it is backed by a small fixed size
//! array which supports the [Copy](std::marker::Copy) trait,
//! hopefully making it easier fast, lock-free concurrent code that
//! deals with commodities.
//!
//! [Currency](Currency) is designed to store useful user-facing
//! information about the currency being referenced by the
//! [CurrencyCode](CurrencyCode), such as its full name/description.
//!
//! ```
//! use commodity::{Commodity, Currency, CurrencyCode};
//! use rust_decimal::Decimal;
//! use std::str::FromStr;
//!
//! // Create a currency from its iso4317 three character code.
//! // The currency stores information associated with the currency,
//! // such as the full name ("United States dollar" for this one).
//! let usd = Currency::from_alpha3("USD").unwrap();
//!
//! // Create a commodity with a value of "2.02 USD"
//! let commodity1 = Commodity::new(Decimal::from_str("2.02").unwrap(), &usd);
//!
//! // Create commodities using the `from_str` method
//! let commodity2 = Commodity::from_str("24.00 USD").unwrap();
//!
//! // Create commodity using a CurrencyCode
//! let nzd_code = CurrencyCode::from_str("NZD").unwrap();
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

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Deserializer};

use arrayvec::ArrayString;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// The length of the [CurrencyCodeArray](CurrencyCodeArray) type,
/// used to store the code/id for a given [Currency](Currency) in
/// [CurrencyCode](CurrencyCode).
pub const CURRENCY_CODE_LENGTH: usize = 8;

/// The type used to store the value of a [CurrencyCode](CurrencyCode).
/// This array backed string has a fixed maximum size 
/// of [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH).
type CurrencyCodeArray = ArrayString<[u8; CURRENCY_CODE_LENGTH]>;

/// An error associated with functionality in the [commodity](./index.html) module.
#[derive(Error, Debug, PartialEq)]
pub enum CommodityError {
    #[error("This commodity {this_commodity:?} is incompatible with {other_commodity:?} because {reason:?}")]
    IncompatableCommodity {
        this_commodity: Commodity,
        other_commodity: Commodity,
        reason: String,
    },
    #[error(
        "The currency code {0} is too long. Maximum of {} characters allowed.",
        CURRENCY_CODE_LENGTH
    )]
    TooLongCurrencyCode(String),
    #[error("The provided alpha3 code {0} doesn't match any in the iso4217 database")]
    InvalidISO4217Alpha3(String),
    #[error("The provided string {0} is invalid, it should be a decimal followed by a currency. e.g. 1.234 USD")]
    InvalidCommodityString(String),
}

/// Represents a the type of currency held in a
/// [Commodity](Commodity). See [CurrencyCode](CurrencyCode) for the
/// primative which is genarally stored and used to refer to a given
/// [Currency](Currency).
#[derive(Debug, Clone)]
pub struct Currency {
    /// Stores the code/id of this currency in a fixed length
    /// [ArrayString](ArrayString), with a maximum length of
    /// [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH).
    pub code: CurrencyCode,
    /// The human readable name of this currency.
    pub name: Option<String>,
}

impl Currency {
    /// Create a new [Currency](Currency)
    ///
    /// # Example
    /// ```
    /// # use commodity::{Currency, CurrencyCode};
    /// use std::str::FromStr;
    ///
    /// let code = CurrencyCode::from_str("AUD").unwrap();
    /// let currency = Currency::new(
    ///     code,
    ///     Some(String::from("Australian Dollar"))
    /// );
    ///
    /// assert_eq!(code, currency.code);
    /// assert_eq!(Some(String::from("Australian Dollar")), currency.name);
    /// ```
    pub fn new(code: CurrencyCode, name: Option<String>) -> Currency {
        Currency { code, name }
    }

    /// Create a [Currency](Currency) from strings, usually for
    /// debugging, or unit testing purposes.
    ///
    /// `code` is an array backed string that has a fixed maximum size
    /// of [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH). The supplied
    /// string must not exeed this, or a
    /// [CommodityError::TooLongCurrencyCode](CommodityError::TooLongCurrencyCode)
    /// will be returned.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Currency, CurrencyCode};
    /// use std::str::FromStr;
    ///
    /// let currency = Currency::from_str("AUD", "Australian dollar").unwrap();
    ///
    /// assert_eq!(CurrencyCode::from_str("AUD").unwrap(), currency.code);
    /// assert_eq!("Australian dollar", currency.name.unwrap());
    /// ```
    pub fn from_str(code: &str, name: &str) -> Result<Currency, CommodityError> {
        let code = CurrencyCode::from_str(code)?;

        let name = if name.len() == 0 {
            None
        } else {
            Some(String::from(name))
        };

        Ok(Currency::new(code, name))
    }

    /// Construct a [Currency](Currency) by looking it up in the iso4217
    /// currency database.
    ///
    /// # Example
    /// ```
    /// # use commodity::Currency;
    ///
    /// let currency = Currency::from_alpha3("AUD").unwrap();
    /// assert_eq!("AUD", currency.code);
    /// assert_eq!(Some(String::from("Australian dollar")), currency.name);
    /// ```
    pub fn from_alpha3(alpha3: &str) -> Result<Currency, CommodityError> {
        match iso4217::alpha3(alpha3) {
            Some(code) => Currency::from_str(alpha3, code.name),
            None => Err(CommodityError::InvalidISO4217Alpha3(String::from(alpha3))),
        }
    }
}

/// Return a vector of all iso4217 currencies
pub fn all_iso4217_currencies() -> Vec<Currency> {
    let mut currencies = Vec::new();
    for iso_currency in iso4217::all() {
        currencies.push(Currency::from_str(iso_currency.alpha3, iso_currency.name).unwrap());
    }

    return currencies;
}

/// The code/id of a [Currency](Currency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurrencyCode {
    /// This is a fixed length array of characters of length [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH),
    /// with a backing implementation based on [ArrayString](ArrayString).
    code_array: CurrencyCodeArray,
}

impl CurrencyCode {
    /// Create a new [CurrencyCode](CurrencyCode).
    pub fn new(code_array: CurrencyCodeArray) -> CurrencyCode {
        CurrencyCode { code_array }
    }
}

impl FromStr for CurrencyCode {
    type Err = CommodityError;

    /// Create a new [Currency](Currency).
    ///
    /// `code` is an array backed string that has a fixed maximum size
    /// of [CURRENCY_CODE_LENGTH](CURRENCY_CODE_LENGTH). The supplied
    /// string must not exeed this, or a
    /// [CommodityError::TooLongCurrencyCode](CommodityError::TooLongCurrencyCode)
    /// will be returned.
    ///
    /// # Example
    /// ```
    /// # use commodity::CurrencyCode;
    /// use std::str::FromStr;
    /// let currency_code = CurrencyCode::from_str("AUD").unwrap();
    /// assert_eq!("AUD", currency_code);
    /// ```
    fn from_str(code: &str) -> Result<CurrencyCode, CommodityError> {
        if code.len() > CURRENCY_CODE_LENGTH {
            return Err(CommodityError::TooLongCurrencyCode(String::from(code)));
        }

        return Ok(CurrencyCode::new(CurrencyCodeArray::from(code).unwrap()));
    }
}

impl From<&Currency> for CurrencyCode {
    fn from(currency: &Currency) -> CurrencyCode {
        currency.code
    }
}

#[cfg(feature = "serde-support")]
impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<CurrencyCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct CurrencyCodeVisitor;

        impl<'de> Visitor<'de> for CurrencyCodeVisitor {
            type Value = CurrencyCode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    format!(
                        "a string with a maximum of {} characters",
                        CURRENCY_CODE_LENGTH
                    )
                    .as_ref(),
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                CurrencyCode::from_str(v).map_err(|e| {
                    E::custom(format!(
                        "there was an error ({}) parsing the currency code string",
                        e
                    ))
                })
            }
        }

        deserializer.deserialize_str(CurrencyCodeVisitor)
    }
}

impl PartialEq<CurrencyCode> for &str {
    fn eq(&self, other: &CurrencyCode) -> bool {
        match CurrencyCodeArray::from_str(self) {
            Ok(self_as_code) => self_as_code == other.code_array,
            Err(_) => false,
        }
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code_array)
    }
}

/// A commodity, which holds a value of a type of [Currrency](Currency)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Commodity {
    pub value: Decimal,
    pub currency_code: CurrencyCode,
}

/// Check whether the currencies of two commodities are compatible (the same),
/// if they aren't then return a [IncompatableCommodity](CurrencyError::IncompatableCommodity) error in the `Result`.
fn check_currency_compatible(
    this_commodity: &Commodity,
    other_commodity: &Commodity,
    reason: String,
) -> Result<(), CommodityError> {
    if !this_commodity.compatible_with(other_commodity) {
        return Err(CommodityError::IncompatableCommodity {
            this_commodity: this_commodity.clone(),
            other_commodity: other_commodity.clone(),
            reason,
        });
    }

    return Ok(());
}

impl Commodity {
    /// Create a new [Commodity](Commodity).
    ///
    /// # Example
    /// 
    /// ```
    /// # use commodity::{Commodity};
    /// use commodity::CurrencyCode;
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), currency_code);
    /// 
    /// assert_eq!(Decimal::from_str("2.02").unwrap(), commodity.value);
    /// assert_eq!(currency_code, commodity.currency_code)
    /// ```
    /// 
    /// Using using the `Into` trait to accept `Currency` as the `currency_code`:
    /// ```
    /// # use commodity::{Commodity};
    /// use std::str::FromStr;
    /// use commodity::Currency;
    /// use rust_decimal::Decimal;
    ///
    /// let currency = Currency::from_alpha3("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), &currency);
    /// ```
    pub fn new<T: Into<CurrencyCode>>(value: Decimal, currency_code: T) -> Commodity {
        Commodity {
            currency_code: currency_code.into(),
            value,
        }
    }

    /// Create a commodity with a value of zero
    pub fn zero(currency_code: CurrencyCode) -> Commodity {
        Commodity::new(Decimal::zero(), currency_code)
    }

    /// Add the value of commodity `other` to `self`
    /// such that `result = self + other`.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), currency_code);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), currency_code);
    ///
    /// // perform the add
    /// let result = commodity1.add(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(650, 2), result.value);
    /// assert_eq!(currency_code, result.currency_code);
    /// ```
    pub fn add(&self, other: &Commodity) -> Result<Commodity, CommodityError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot add commodities with different currencies"),
        )?;

        return Ok(Commodity::new(self.value + other.value, self.currency_code));
    }

    /// Subtract the value of commodity `other` from `self`
    /// such that `result = self - other`.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), currency_code);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), currency_code);
    ///
    /// // perform the subtraction
    /// let result = commodity1.sub(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(150, 2), result.value);
    /// assert_eq!(currency_code, result.currency_code);
    /// ```
    pub fn sub(&self, other: &Commodity) -> Result<Commodity, CommodityError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot subtract commodities with different currencies"),
        )?;

        return Ok(Commodity::new(self.value - other.value, self.currency_code));
    }

    /// Negate the value of this commodity such that `result = -self`
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CurrencyCode};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let currency_code = CurrencyCode::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), currency_code);
    ///
    /// // perform the negation
    /// let result = commodity.neg();
    ///
    /// assert_eq!(Decimal::from_str("-2.02").unwrap(), result.value);
    /// assert_eq!(currency_code, result.currency_code)
    /// ```
    pub fn neg(&self) -> Commodity {
        Commodity::new(-self.value, self.currency_code)
    }

    /// Divide this commodity by the specified integer value
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use rust_decimal::{Decimal};
    /// use std::str::FromStr;
    ///
    /// let commodity = Commodity::from_str("4.03 AUD").unwrap();
    /// let result = commodity.div_i64(4);
    /// assert_eq!(Decimal::new(10075, 4), result.value);
    /// ```
    pub fn div_i64(&self, i: i64) -> Commodity {
        let decimal = Decimal::new(i * 100, 2);
        Commodity::new(self.value / decimal, self.currency_code)
    }

    /// Divide this commodity by the specified integer value
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use rust_decimal::{Decimal};
    /// use std::str::FromStr;
    ///
    /// let commodity = Commodity::from_str("4.03 AUD").unwrap();
    /// let results = commodity.divide_share(4, 2);
    ///
    /// assert_eq!(Decimal::new(101, 2), results.get(0).unwrap().value);
    /// assert_eq!(Decimal::new(101, 2), results.get(1).unwrap().value);
    /// assert_eq!(Decimal::new(101, 2), results.get(2).unwrap().value);
    /// assert_eq!(Decimal::new(100, 2), results.get(3).unwrap().value);
    /// ```
    pub fn divide_share(&self, i: i64, dp: u32) -> Vec<Commodity> {
        // TODO: rework this algorithm
        // 
        // Consider the following idea:
        // Use the normal divide, then round it. Sum it up, and
        // subtract this from the original number, to get the
        // remainder. Add the remainder one digit at a time to the
        // resulting shares.

        let mut commodities: Vec<Commodity> = Vec::new();
        let divisor = Decimal::new(i * 10_i64.pow(dp), dp);
        let remainder = self.value % divisor;
        // = 0.03

        let divided = self.value / divisor;
        // 4.03 / 0.04 = 100.75
        // divided.set_scale(dp * 2).unwrap();
        // = 1.0075
        let truncated = divided.trunc();
        // = 1.00

        let dp_divisor = Decimal::new(1, dp);

        let remainder_bits = (remainder / dp_divisor).to_i64().unwrap();
        let remainder_bits_abs = remainder_bits.abs();
        let i_abs = i.abs();

        // dbg!(self.value);
        // dbg!(i);
        // dbg!(divided);
        // dbg!(truncated);
        // dbg!(remainder_bits);
        // dbg!(remainder);

        let sign = Decimal::new(remainder_bits.signum() * i.signum(), 0);

        for commodity_index in 1..=i_abs {
            let value = if commodity_index <= remainder_bits_abs {
                truncated + dp_divisor * sign
            } else {
                truncated
            };

            commodities.push(Commodity::new(value, self.currency_code))
        }

        dbg!(commodities.clone());

        return commodities;
    }

    /// Convert this commodity to a different currency using a conversion rate.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CurrencyCode};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let aud = Commodity::from_str("100.00 AUD").unwrap();
    /// let usd = aud.convert(CurrencyCode::from_str("USD").unwrap(), Decimal::from_str("0.01").unwrap());
    ///
    /// assert_eq!(Decimal::from_str("1.00").unwrap(), usd.value);
    /// assert_eq!("USD", usd.currency_code);
    /// ```
    pub fn convert(&self, currency_code: CurrencyCode, rate: Decimal) -> Commodity {
        Commodity::new(self.value * rate, currency_code)
    }

    /// Returns true if the currencies of both this commodity, and
    /// the `other` commodity are compatible for numeric operations.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use std::str::FromStr;
    /// 
    /// let aud1 = Commodity::from_str("1.0 AUD").unwrap();
    /// let aud2 = Commodity::from_str("2.0 AUD").unwrap();
    /// let nzd = Commodity::from_str("1.0 NZD").unwrap();
    ///
    /// assert!(aud1.compatible_with(&aud2));
    /// assert!(!aud1.compatible_with(&nzd));
    /// ```
    pub fn compatible_with(&self, other: &Commodity) -> bool {
        return self.currency_code == other.currency_code;
    }

    /// Compare whether this commodity has a value less than another commodity.
    ///
    /// Will return an error if the commodities have incompatible currencies.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use std::str::FromStr;
    /// 
    /// let aud1 = Commodity::from_str("1.0 AUD").unwrap();
    /// let aud2 = Commodity::from_str("2.0 AUD").unwrap();
    ///
    /// assert_eq!(true, aud1.lt(&aud2).unwrap());
    /// assert_eq!(false, aud2.lt(&aud1).unwrap());
    /// ```
    pub fn lt(&self, other: &Commodity) -> Result<bool, CommodityError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot compare commodities with different currencies"),
        )?;

        Ok(self.value < other.value)
    }

    /// Compare whether this commodity has a value greater than another commodity.
    ///
    /// Will return an error if the commodities have incompatible currencies.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use std::str::FromStr;
    /// 
    /// let aud1 = Commodity::from_str("1.0 AUD").unwrap();
    /// let aud2 = Commodity::from_str("2.0 AUD").unwrap();
    ///
    /// assert_eq!(false, aud1.gt(&aud2).unwrap());
    /// assert_eq!(true, aud2.gt(&aud1).unwrap());
    /// ```
    pub fn gt(&self, other: &Commodity) -> Result<bool, CommodityError> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot compare commodities with different currencies"),
        )?;

        Ok(self.value > other.value)
    }

    /// Return the absolute value of this commodity (if the value is
    /// negative, then make it positive).
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity};
    /// use std::str::FromStr;
    /// 
    /// let aud1 = Commodity::from_str("-1.0 AUD").unwrap();
    /// assert_eq!(Commodity::from_str("1.0 AUD").unwrap(), aud1.abs());
    ///
    /// let aud2 = Commodity::from_str("2.0 AUD").unwrap();
    /// assert_eq!(Commodity::from_str("2.0 AUD").unwrap(), aud2.abs());
    /// ```
    pub fn abs(&self) -> Commodity {
        return Commodity::new(self.value.abs(), self.currency_code);
    }

    /// The default epsilon to use for comparisons between different [Commodity](Commodity)s.
    pub fn default_epsilon() -> Decimal {
        Decimal::new(1, 6)
    }
    pub fn eq_approx(&self, other: Commodity, epsilon: Decimal) -> bool {
        if other.currency_code != self.currency_code {
            return false;
        }

        let diff = if self.value > other.value {
            self.value - other.value
        } else {
            other.value - self.value
        };

        return diff <= epsilon;
    }
}

impl FromStr for Commodity {
    type Err = CommodityError;
    /// Construct a [Commodity](Commodity) from a string
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CurrencyCode};
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let commodity = Commodity::from_str("1.234 USD").unwrap();
    ///
    /// assert_eq!(Decimal::from_str("1.234").unwrap(), commodity.value);
    /// assert_eq!(CurrencyCode::from_str("USD").unwrap(), commodity.currency_code);
    /// ```
    fn from_str(commodity_string: &str) -> Result<Commodity, CommodityError> {
        let elements: Vec<&str> = commodity_string.split_whitespace().collect();

        if elements.len() != 2 {
            return Err(CommodityError::InvalidCommodityString(String::from(
                commodity_string,
            )));
        }

        Ok(Commodity::new(
            Decimal::from_str(elements.get(0).unwrap()).unwrap(),
            CurrencyCode::from_str(elements.get(1).unwrap())?,
        ))
    }
}

impl PartialOrd for Commodity {
    fn partial_cmp(&self, other: &Commodity) -> Option<std::cmp::Ordering> {
        check_currency_compatible(
            self,
            other,
            String::from("cannot compare commodities with different currencies"),
        )
        .unwrap();

        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Commodity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        check_currency_compatible(
            self,
            other,
            String::from("cannot compare commodities with different currencies"),
        )
        .unwrap();

        self.value.cmp(&other.value)
    }
}

impl fmt::Display for Commodity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency_code)
    }
}

#[cfg(test)]
mod tests {
    use super::{Commodity, CurrencyCode, CommodityError};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // #[test]
    // fn divide_larger() {
    //     let commodity = Commodity::from_str("4.25 AUD").unwrap();
    //     let results = commodity.divide_share(4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(107, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(106, 2), results.get(3).unwrap().value);
    // }

    // #[test]
    // fn divide_share_negative_dividend() {
    //     let commodity = Commodity::from_str("-4.03 AUD").unwrap();
    //     let results = commodity.divide_share(4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(-101, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(-100, 2), results.get(3).unwrap().value);
    // }

    // #[test]
    // fn divide_share_negative_divisor() {
    //     let commodity = Commodity::from_str("4.03 AUD").unwrap();
    //     let results = commodity.divide_share(-4, 2);

    //     assert_eq!(4, results.len());
    //     assert_eq!(Decimal::new(-101, 2), results.get(0).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(1).unwrap().value);
    //     assert_eq!(Decimal::new(-101, 2), results.get(2).unwrap().value);
    //     assert_eq!(Decimal::new(-100, 2), results.get(3).unwrap().value);
    // }

    #[test]
    fn commodity_incompatible_currency() {
        let currency1 = CurrencyCode::from_str("USD").unwrap();
        let currency2 = CurrencyCode::from_str("AUD").unwrap();

        let commodity1 = Commodity::new(Decimal::new(400, 2), currency1);
        let commodity2 = Commodity::new(Decimal::new(250, 2), currency2);

        let error1 = commodity1.add(&commodity2).expect_err("expected an error");

        assert_eq!(
            CommodityError::IncompatableCommodity {
                this_commodity: commodity1.clone(),
                other_commodity: commodity2.clone(),
                reason: String::from("cannot add commodities with different currencies"),
            },
            error1
        );

        let error2 = commodity1.sub(&commodity2).expect_err("expected an error");

        assert_eq!(
            CommodityError::IncompatableCommodity {
                this_commodity: commodity1,
                other_commodity: commodity2,
                reason: String::from("cannot subtract commodities with different currencies"),
            },
            error2
        );
    }
}

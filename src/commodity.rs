use arrayvec::ArrayString;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::{hash::Hash, str::FromStr};
use thiserror::Error;

/// The length of the [CommodityTypeIDArray](CommodityTypeIDArray) type,
/// used to store the id for a given [CommodityType](CommodityType) in
/// [CommodityTypeID](CommodityTypeID).
pub const COMMODITY_TYPE_ID_LENGTH: usize = 8;

/// The type used to store the value of a [CommodityTypeID](CommodityTypeID).
/// This array backed string has a fixed maximum size
/// of [COMMODITY_TYPE_ID_LENGTH](COMMODITY_TYPE_ID_LENGTH).
type CommodityTypeIDArray = ArrayString<[u8; COMMODITY_TYPE_ID_LENGTH]>;

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
        "The commodity id {0} is too long. Maximum of {} characters allowed.",
        COMMODITY_TYPE_ID_LENGTH
    )]
    TooLongCommodityTypeID(String),
    #[cfg(feature = "iso4217")]
    #[error("The provided alpha3 code {0} doesn't match any in the iso4217 database")]
    #[cfg(feature = "iso4217")]
    InvalidISO4217Alpha3(String),
    #[error("The provided string {0} is invalid, it should be a decimal followed by a commodity_type. e.g. 1.234 USD")]
    InvalidCommodityString(String),
}

/// Represents a type of [Commodity](Commodity).
///
/// See [CommodityTypeID](CommodityTypeID) for the primative which is
/// genarally stored and used to refer to a given
/// [CommodityType](CommodityType).
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq)]
pub struct CommodityType {
    /// Stores the id of this commodity type in a fixed length
    /// [ArrayString](ArrayString), with a maximum length of
    /// [COMMODITY_TYPE_ID_LENGTH](COMMODITY_TYPE_ID_LENGTH).
    pub id: CommodityTypeID,
    /// The human readable name of this commodity_type.
    pub name: Option<String>,
}

impl CommodityType {
    /// Create a new [CommodityType](CommodityType).
    ///
    /// # Example
    /// ```
    /// # use commodity::{CommodityType, CommodityTypeID};
    /// use std::str::FromStr;
    ///
    /// let id = CommodityTypeID::from_str("AUD").unwrap();
    /// let commodity_type = CommodityType::new(
    ///     id,
    ///     Some(String::from("Australian Dollar"))
    /// );
    ///
    /// assert_eq!(id, commodity_type.id);
    /// assert_eq!(Some(String::from("Australian Dollar")), commodity_type.name);
    /// ```
    pub fn new(id: CommodityTypeID, name: Option<String>) -> CommodityType {
        CommodityType { id, name }
    }

    /// Create a [CommodityType](CommodityType) from strings, usually
    /// for debugging, or unit testing purposes.
    ///
    /// `id` is an array backed string that has a fixed maximum size
    /// of [COMMODITY_TYPE_ID_LENGTH](COMMODITY_TYPE_ID_LENGTH). The supplied
    /// string must not exeed this, or a
    /// [CommodityError::TooLongCommodityTypeID](CommodityError::TooLongCommodityTypeID)
    /// will be returned.
    ///
    /// # Example
    /// ```
    /// # use commodity::{CommodityType, CommodityTypeID};
    /// use std::str::FromStr;
    ///
    /// let commodity_type = CommodityType::from_str("AUD", "Australian dollar").unwrap();
    ///
    /// assert_eq!(CommodityTypeID::from_str("AUD").unwrap(), commodity_type.id);
    /// assert_eq!("Australian dollar", commodity_type.name.unwrap());
    /// ```
    pub fn from_str<SR: AsRef<str>, SI: Into<String>>(
        id: SR,
        name: SI,
    ) -> Result<CommodityType, CommodityError> {
        let id = CommodityTypeID::from_str(id.as_ref())?;
        let name_string: String = name.into();

        let name_option = if name_string.is_empty() {
            None
        } else {
            Some(name_string)
        };

        Ok(CommodityType::new(id, name_option))
    }

    /// Construct a [CommodityType](CommodityType) by looking it up in the `ISO4217`
    /// currencies database.
    ///
    /// # Example
    /// ```
    /// # use commodity::CommodityType;
    ///
    /// let commodity_type = CommodityType::from_currency_alpha3("AUD").unwrap();
    /// assert_eq!("AUD", commodity_type.id);
    /// assert_eq!(Some(String::from("Australian dollar")), commodity_type.name);
    /// ```
    #[cfg(feature = "iso4217")]
    pub fn from_currency_alpha3<S: AsRef<str>>(alpha3: S) -> Result<CommodityType, CommodityError> {
        match iso4217::alpha3(alpha3.as_ref()) {
            Some(id) => CommodityType::from_str(alpha3, id.name),
            None => Err(CommodityError::InvalidISO4217Alpha3(String::from(
                alpha3.as_ref(),
            ))),
        }
    }
}

/// This implementation only checks that the ids match. It assumes
/// that you will not have logical different commodity types with the
/// same id but different names.
impl PartialEq for CommodityType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for CommodityType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl fmt::Display for CommodityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} ({})", self.id, name),
            None => write!(f, "{}", self.id),
        }
    }
}

/// Return a vector of all `ISO4217` currencies
#[cfg(feature = "iso4217")]
pub fn all_iso4217_currencies() -> Vec<CommodityType> {
    let mut currencies = Vec::new();
    for iso_commodity_type in iso4217::all() {
        currencies.push(
            CommodityType::from_str(iso_commodity_type.alpha3, iso_commodity_type.name).unwrap(),
        );
    }

    currencies
}

/// The id of a [CommodityType](CommodityType) stored in a fixed
/// length array using [ArrayString](ArrayString), with a maximum
/// length of [COMMODITY_TYPE_ID_LENGTH](COMMODITY_TYPE_ID_LENGTH).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommodityTypeID {
    id_array: CommodityTypeIDArray,
}

impl CommodityTypeID {
    /// Create a new [CommodityTypeID](CommodityTypeID).
    pub fn new(id_array: CommodityTypeIDArray) -> CommodityTypeID {
        CommodityTypeID { id_array }
    }
}

impl FromStr for CommodityTypeID {
    type Err = CommodityError;

    /// Create a new [CommodityType](CommodityType).
    ///
    /// `code` is an array backed string that has a fixed maximum size
    /// of [COMMODITY_TYPE_ID_LENGTH](COMMODITY_TYPE_ID_LENGTH). The supplied
    /// string must not exeed this, or a
    /// [CommodityError::TooLongCommodityTypeID](CommodityError::TooLongCommodityTypeID)
    /// will be returned.
    ///
    /// # Example
    /// ```
    /// # use commodity::CommodityTypeID;
    /// use std::str::FromStr;
    /// let commodity_id = CommodityTypeID::from_str("AUD").unwrap();
    /// assert_eq!("AUD", commodity_id);
    /// ```
    fn from_str(id: &str) -> Result<CommodityTypeID, CommodityError> {
        if id.len() > COMMODITY_TYPE_ID_LENGTH {
            return Err(CommodityError::TooLongCommodityTypeID(String::from(id)));
        }

        Ok(CommodityTypeID::new(
            CommodityTypeIDArray::from(id).unwrap(),
        ))
    }
}

impl From<&CommodityType> for CommodityTypeID {
    fn from(commodity_type: &CommodityType) -> CommodityTypeID {
        commodity_type.id
    }
}

#[cfg(feature = "serde-support")]
impl<'de> Deserialize<'de> for CommodityTypeID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<CommodityTypeID, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct CommodityTypeIDVisitor;

        impl<'de> Visitor<'de> for CommodityTypeIDVisitor {
            type Value = CommodityTypeID;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    format!(
                        "a string with a maximum of {} characters",
                        COMMODITY_TYPE_ID_LENGTH
                    )
                    .as_ref(),
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                CommodityTypeID::from_str(v).map_err(|e| {
                    E::custom(format!(
                        "there was an error ({}) parsing the commodity_type id string",
                        e
                    ))
                })
            }
        }

        deserializer.deserialize_str(CommodityTypeIDVisitor)
    }
}

#[cfg(feature = "serde-support")]
impl Serialize for CommodityTypeID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id_array)
    }
}

impl PartialEq<CommodityTypeID> for &str {
    fn eq(&self, other: &CommodityTypeID) -> bool {
        match CommodityTypeIDArray::from_str(self) {
            Ok(self_as_code) => self_as_code == other.id_array,
            Err(_) => false,
        }
    }
}

impl fmt::Display for CommodityTypeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id_array)
    }
}

/// A commodity, which holds a value with an associated [CommodityType](CommodityType)
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Commodity {
    /// The value of this commodity
    pub value: Decimal,
    /// The id of the type of this commodity
    pub type_id: CommodityTypeID,
}

/// Check whether the currencies of two commodities are compatible (the same),
/// if they aren't then return a [IncompatableCommodity](CommodityTypeError::IncompatableCommodity) error in the `Result`.
fn check_commodity_type_compatible(
    this_commodity: &Commodity,
    other_commodity: &Commodity,
    reason: String,
) -> Result<(), CommodityError> {
    if !this_commodity.compatible_with(other_commodity) {
        return Err(CommodityError::IncompatableCommodity {
            this_commodity: *this_commodity,
            other_commodity: *other_commodity,
            reason,
        });
    }

    Ok(())
}

impl Commodity {
    /// Create a new [Commodity](Commodity).
    ///
    /// # Example
    ///
    /// ```
    /// use commodity::{Commodity, CommodityTypeID};
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let type_id = CommodityTypeID::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), type_id);
    ///
    /// assert_eq!(Decimal::from_str("2.02").unwrap(), commodity.value);
    /// assert_eq!(type_id, commodity.type_id)
    /// ```
    ///
    /// Using using the `Into` trait to accept `CommodityType` as the `type_id`:
    /// ```
    /// use commodity::{Commodity, CommodityType};
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let commodity_type = CommodityType::from_currency_alpha3("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), &commodity_type);
    /// ```
    pub fn new<T: Into<CommodityTypeID>>(value: Decimal, type_id: T) -> Commodity {
        Commodity {
            type_id: type_id.into(),
            value,
        }
    }

    /// Create a commodity with a value of zero
    pub fn zero(type_id: CommodityTypeID) -> Commodity {
        Commodity::new(Decimal::zero(), type_id)
    }

    /// Add the value of commodity `other` to `self`
    /// such that `result = self + other`.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CommodityTypeID};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let type_id = CommodityTypeID::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), type_id);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), type_id);
    ///
    /// // perform the add
    /// let result = commodity1.add(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(650, 2), result.value);
    /// assert_eq!(type_id, result.type_id);
    /// ```
    pub fn add(&self, other: &Commodity) -> Result<Commodity, CommodityError> {
        check_commodity_type_compatible(
            self,
            other,
            String::from("cannot add commodities with different currencies"),
        )?;

        Ok(Commodity::new(self.value + other.value, self.type_id))
    }

    /// Subtract the value of commodity `other` from `self`
    /// such that `result = self - other`.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CommodityTypeID};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let usd = CommodityTypeID::from_str("USD").unwrap();
    /// let commodity1 = Commodity::new(Decimal::new(400, 2), usd);
    /// let commodity2 = Commodity::new(Decimal::new(250, 2), usd);
    ///
    /// // perform the subtraction
    /// let result = commodity1.sub(&commodity2).unwrap();
    ///
    /// assert_eq!(Decimal::new(150, 2), result.value);
    /// assert_eq!(usd, result.type_id);
    /// ```
    pub fn sub(&self, other: &Commodity) -> Result<Commodity, CommodityError> {
        check_commodity_type_compatible(
            self,
            other,
            String::from("cannot subtract commodities with different currencies"),
        )?;

        Ok(Commodity::new(self.value - other.value, self.type_id))
    }

    /// Negate the value of this commodity such that `result = -self`
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CommodityTypeID};
    /// # use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let type_id = CommodityTypeID::from_str("USD").unwrap();
    /// let commodity = Commodity::new(Decimal::new(202, 2), type_id);
    ///
    /// // perform the negation
    /// let result = commodity.neg();
    ///
    /// assert_eq!(Decimal::from_str("-2.02").unwrap(), result.value);
    /// assert_eq!(type_id, result.type_id)
    /// ```
    pub fn neg(&self) -> Commodity {
        Commodity::new(-self.value, self.type_id)
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
        Commodity::new(self.value / decimal, self.type_id)
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

            commodities.push(Commodity::new(value, self.type_id))
        }

        commodities
    }

    /// Convert this commodity to a different commodity_type using a conversion rate.
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CommodityTypeID};
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    ///
    /// let aud = Commodity::from_str("100.00 AUD").unwrap();
    /// let usd = aud.convert(CommodityTypeID::from_str("USD").unwrap(), Decimal::from_str("0.01").unwrap());
    ///
    /// assert_eq!(Decimal::from_str("1.00").unwrap(), usd.value);
    /// assert_eq!("USD", usd.type_id);
    /// ```
    pub fn convert(&self, type_id: CommodityTypeID, rate: Decimal) -> Commodity {
        Commodity::new(self.value * rate, type_id)
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
        self.type_id == other.type_id
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
        check_commodity_type_compatible(
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
        check_commodity_type_compatible(
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
        Commodity::new(self.value.abs(), self.type_id)
    }

    /// The default epsilon to use for comparisons between different [Commodity](Commodity)s.
    pub fn default_epsilon() -> Decimal {
        Decimal::new(1, 6)
    }
    pub fn eq_approx(&self, other: Commodity, epsilon: Decimal) -> bool {
        if other.type_id != self.type_id {
            return false;
        }

        let diff = if self.value > other.value {
            self.value - other.value
        } else {
            other.value - self.value
        };

        diff <= epsilon
    }
}

impl FromStr for Commodity {
    type Err = CommodityError;
    /// Construct a [Commodity](Commodity) from a string
    ///
    /// # Example
    /// ```
    /// # use commodity::{Commodity, CommodityTypeID};
    /// use std::str::FromStr;
    /// use rust_decimal::Decimal;
    ///
    /// let commodity = Commodity::from_str("1.234 USD").unwrap();
    ///
    /// assert_eq!(Decimal::from_str("1.234").unwrap(), commodity.value);
    /// assert_eq!(CommodityTypeID::from_str("USD").unwrap(), commodity.type_id);
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
            CommodityTypeID::from_str(elements.get(1).unwrap())?,
        ))
    }
}

impl PartialOrd for Commodity {
    fn partial_cmp(&self, other: &Commodity) -> Option<std::cmp::Ordering> {
        check_commodity_type_compatible(
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
        check_commodity_type_compatible(
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
        write!(f, "{} {}", self.value, self.type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{Commodity, CommodityError, CommodityType, CommodityTypeID};
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
    fn commodity_incompatible_commodity_type() {
        let commodity_type1 = CommodityTypeID::from_str("USD").unwrap();
        let commodity_type2 = CommodityTypeID::from_str("AUD").unwrap();

        let commodity1 = Commodity::new(Decimal::new(400, 2), commodity_type1);
        let commodity2 = Commodity::new(Decimal::new(250, 2), commodity_type2);

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

    #[cfg(feature = "serde-support")]
    #[test]
    fn test_type_id_serialization() {
        use serde_json;

        let original_data = "\"AUD\"";
        let type_id: CommodityTypeID = serde_json::from_str(original_data).unwrap();

        assert_eq!(CommodityTypeID::from_str("AUD").unwrap(), type_id);

        let serialized_data = serde_json::to_string(&type_id).unwrap();
        assert_eq!(original_data, serialized_data);
    }

    #[cfg(feature = "serde-support")]
    #[test]
    fn test_commodity_serialization() {
        use serde_json;

        let original_data = r#"{
  "value": "1.0",
  "type_id": "AUD"
}"#;
        let type_id: Commodity = serde_json::from_str(original_data).unwrap();

        assert_eq!(Commodity::from_str("1.0 AUD").unwrap(), type_id);

        let serialized_data = serde_json::to_string_pretty(&type_id).unwrap();
        assert_eq!(original_data, serialized_data);
    }

    /// Test the `PartialEq` implementation for `CommodityType`.
    #[test]
    fn test_commodity_type_partial_eq() {
        let aud = CommodityType::from_str("AUD", "Australian Dollar").unwrap();
        let aud2 = CommodityType::from_str("AUD", "Australian Dollar 2").unwrap();
        assert!(aud == aud2);

        let usd = CommodityType::from_str("USD", "United States Dollar").unwrap();
        assert!(aud != usd);
    }

    /// Test the `Display` implementation for `CommodityType`.
    #[test]
    fn test_commodity_type_display() {
        let aud = CommodityType::from_str("AUD", "Australian dollar").unwrap();
        assert_eq!("AUD (Australian dollar)", &format!("{}", aud));

        let test = CommodityType::new(CommodityTypeID::from_str("TEST").unwrap(), None);
        assert_eq!("TEST", &format!("{}", test))
    }
}

//! Types and utilities relating to exchange rates and conversions
//! between different types of commodities.

use crate::{Commodity, CommodityTypeID};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use thiserror::Error;

/// An error associated with functionality in the [exchange_rate](crate::exchange_rate) module.
#[derive(Error, Debug)]
pub enum ExchangeRateError {
    #[error("The commodity type with id {0} is not present in the exchange rate.")]
    CommodityTypeNotPresent(CommodityTypeID),
    #[error("There was a divide overflow while computing the exchange rate, performing the division {0}/{1}.")]
    DivideOverflow(Decimal, Decimal),
}

/// Represents the exchange rate between [Commodity](Commodity)s
/// with different [CommodityType](crate::CommodityType)s.
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeRate {
    /// The datetime that this exchange rate represents
    pub date: Option<NaiveDate>,
    /// The datetime that this exchange rate was obtained.
    pub obtained_datetime: Option<DateTime<Utc>>,
    /// The id of the base commodity type for the exchange rate
    pub base: Option<CommodityTypeID>,
    /// Maps commodity type ids, to the conversion rate from that
    /// [CommodityType](crate::CommodityType) to the `base`
    /// [CommodityType](crate::CommodityType).
    pub rates: BTreeMap<CommodityTypeID, Decimal>,
}

impl ExchangeRate {
    pub fn get_rate(&self, commodity_type_id: &CommodityTypeID) -> Option<&Decimal> {
        self.rates.get(commodity_type_id)
    }

    /// Convert the [CommodityType](crate::CommodityType) of a
    /// [Commodity](Commodity) to another
    /// [CommodityType](crate::CommodityType) using this
    /// [ExchangeRate](ExchangeRate).
    pub fn convert(
        &self,
        commodity: Commodity,
        target_commodity_type: CommodityTypeID,
    ) -> Result<Commodity, ExchangeRateError> {
        if let Some(base) = self.base {
            if commodity.type_id == base {
                if let Some(rate) = self.get_rate(&target_commodity_type) {
                    return Ok(Commodity::new(
                        rate * commodity.value,
                        target_commodity_type,
                    ));
                };
            }

            if target_commodity_type == base {
                if let Some(rate) = self.get_rate(&commodity.type_id) {
                    let div = commodity
                        .value
                        .checked_div(*rate)
                        .ok_or_else(|| ExchangeRateError::DivideOverflow(commodity.value, *rate))?;
                    return Ok(Commodity::new(div, target_commodity_type));
                };
            }
        }

        // handle the situation where there is no base commodity type, or neither the commodity
        // type or the target commodity type are the base commodity type.

        let commodity_rate = match self.get_rate(&commodity.type_id) {
            Some(rate) => rate,
            None => {
                return Err(ExchangeRateError::CommodityTypeNotPresent(
                    commodity.type_id,
                ))
            }
        };

        let target_rate = match self.get_rate(&target_commodity_type) {
            Some(rate) => rate,
            None => {
                return Err(ExchangeRateError::CommodityTypeNotPresent(
                    target_commodity_type,
                ))
            }
        };

        let div = commodity
            .value
            .checked_div(*commodity_rate)
            .ok_or_else(|| ExchangeRateError::DivideOverflow(commodity.value, *commodity_rate))?;
        let value = div * target_rate;

        Ok(Commodity::new(value, target_commodity_type))
    }

    /// Get the exchange rate between two commodity types present in this exchange
    /// rate data structure. Returns `None` if one of the commodity types is not present.
    pub fn rate_between(
        &self,
        from: &CommodityTypeID,
        to: &CommodityTypeID,
    ) -> Result<Option<Decimal>, ExchangeRateError> {
        if let Some(base) = &self.base {
            if from == base {
                if let Some(rate) = self.get_rate(&to) {
                    return Ok(Some(*rate));
                };
            }

            if to == base {
                if let Some(rate) = self.get_rate(&from) {
                    let one = Decimal::new(1, 0);
                    return match one.checked_div(*rate) {
                        Some(value) => Ok(Some(value)),
                        None => Err(ExchangeRateError::DivideOverflow(one, *rate)),
                    };
                };
            }
        }

        // handle the situation where there is no base commodity type, or neither the from
        // or the to commodity types are the base commodity type.

        let from_rate = match self.get_rate(&from) {
            Some(rate) => rate,
            None => return Ok(None),
        };

        let to_rate = match self.get_rate(&to) {
            Some(rate) => rate,
            None => return Ok(None),
        };

        match to_rate.checked_div(*from_rate) {
            Some(value) => Ok(Some(value)),
            None => Err(ExchangeRateError::DivideOverflow(*to_rate, *from_rate)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Commodity, CommodityTypeID, ExchangeRate};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    #[cfg(feature = "serde-support")]
    #[test]
    fn test_json_serialization() {
        use serde_json;

        let original_data = r#"{
    "date": "2020-02-07",
    "base": "AUD",
    "rates": {
        "USD": 2.542,
        "EU": "1.234"
    }
}
"#;

        let exchange_rate: ExchangeRate = serde_json::from_str(original_data).unwrap();
        let usd = CommodityTypeID::from_str("USD").unwrap();
        let eu = CommodityTypeID::from_str("EU").unwrap();

        assert_eq!(
            NaiveDate::from_ymd(2020, 02, 07),
            exchange_rate.date.unwrap()
        );
        assert_eq!("AUD", exchange_rate.base.unwrap());
        assert_eq!(
            Decimal::from_str("2.542").unwrap(),
            *exchange_rate.get_rate(&usd).unwrap()
        );
        assert_eq!(
            Decimal::from_str("1.234").unwrap(),
            *exchange_rate.get_rate(&eu).unwrap()
        );

        let expected_serialized_data = r#"{
  "date": "2020-02-07",
  "obtained_datetime": null,
  "base": "AUD",
  "rates": {
    "EU": "1.234",
    "USD": "2.542"
  }
}"#;

        let serialized_data = serde_json::to_string_pretty(&exchange_rate).unwrap();
        assert_eq!(expected_serialized_data, serialized_data);
    }

    /// Convert between two commodities at a reference rate (no base rate commodity type).
    #[test]
    fn convert_reference_rates() {
        let mut rates: BTreeMap<CommodityTypeID, Decimal> = BTreeMap::new();
        let aud = CommodityTypeID::from_str("AUD").unwrap();
        let nzd = CommodityTypeID::from_str("NZD").unwrap();
        rates.insert(aud, Decimal::from_str("1.6417").unwrap());
        rates.insert(nzd, Decimal::from_str("1.7094").unwrap());

        let exchange_rate = ExchangeRate {
            date: Some(NaiveDate::from_ymd(2020, 02, 07)),
            base: None,
            obtained_datetime: None,
            rates,
        };

        {
            let start_commodity = Commodity::new(Decimal::from_str("10.0").unwrap(), aud);
            let converted_commodity = exchange_rate.convert(start_commodity, nzd);
            assert_eq!(
                Decimal::from_str("10.412377413656575501005055735").unwrap(),
                converted_commodity.unwrap().value
            );
            assert_eq!(
                exchange_rate.rate_between(&aud, &nzd).unwrap(),
                Some(Decimal::from_str("1.0412377413656575501005055734").unwrap())
            );
        }

        {
            let start_commodity = Commodity::new(Decimal::from_str("10.0").unwrap(), nzd);
            let converted_commodity = exchange_rate.convert(start_commodity, aud);
            assert_eq!(
                Decimal::from_str("9.603954603954603954603954604").unwrap(),
                converted_commodity.unwrap().value
            );
            assert_eq!(
                exchange_rate.rate_between(&nzd, &aud).unwrap(),
                Some(Decimal::from_str("0.9603954603954603954603954603").unwrap())
            );
        }
    }

    /// Convert between commodities using an exchange rate with a base rate.
    #[test]
    fn convert_base_rate() {
        let mut rates: BTreeMap<CommodityTypeID, Decimal> = BTreeMap::new();
        let nok = CommodityTypeID::from_str("NOK").unwrap();
        let usd = CommodityTypeID::from_str("USD").unwrap();
        let gel = CommodityTypeID::from_str("GEL").unwrap();

        rates.insert(nok, Decimal::from_str("9.2691220713").unwrap());
        rates.insert(gel, Decimal::from_str("3.08").unwrap());

        let exchange_rate = ExchangeRate {
            date: Some(NaiveDate::from_ymd(2020, 02, 07)),
            base: Some(usd),
            obtained_datetime: None,
            rates,
        };

        {
            let start_commodity = Commodity::new(Decimal::from_str("100.0").unwrap(), usd);
            let converted_commodity = exchange_rate.convert(start_commodity, nok);
            assert_eq!(
                Decimal::from_str("926.91220713000").unwrap(),
                converted_commodity.unwrap().value
            );
            assert_eq!(
                exchange_rate.rate_between(&usd, &nok).unwrap(),
                Some(Decimal::from_str("9.2691220713").unwrap())
            );
        }

        {
            let start_commodity = Commodity::new(Decimal::from_str("100.0").unwrap(), nok);
            let converted_commodity = exchange_rate.convert(start_commodity, usd);
            assert_eq!(
                Decimal::from_str("10.788508256853169187585300627").unwrap(),
                converted_commodity.unwrap().value
            );
            assert_eq!(
                exchange_rate.rate_between(&nok, &usd).unwrap(),
                Some(Decimal::from_str("0.1078850825685316918758530063").unwrap())
            );
        }

        {
            let start_commodity = Commodity::new(Decimal::from_str("100.0").unwrap(), nok);
            let converted_commodity = exchange_rate.convert(start_commodity, gel);
            assert_eq!(
                Decimal::from_str("33.228605431107761097762725931").unwrap(),
                converted_commodity.unwrap().value
            );
            assert_eq!(
                exchange_rate.rate_between(&nok, &gel).unwrap(),
                Some(Decimal::from_str("0.3322860543110776109776272593").unwrap())
            );
        }
    }
}

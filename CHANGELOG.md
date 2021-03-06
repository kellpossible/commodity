# Changelog for Commodity

## v0.4.0

New Features:

+ Add `rate_between()` method to `ExchangeRate`.

Breaking:

+ Make `iso4217` an optional dependency (disabled by default).

## v0.3.7

+ Implement `Hash` for `CommodityType`.

## v0.3.6

+ Implement `Eq` for `CommodityType`.

## v0.3.5

+ Implement `PartialEq` for `ExchangeRate`.

## v0.3.4

+ Implement `Display` for `CommodityType`.

## v0.3.3

+ Implement `PartialEq` for `CommodityType`.

## v0.3.2

+ Fix this changelog!

## v0.3.1

+ Bump `rust_decimal` dependency up to using generic version `1` to address [#3](https://github.com/kellpossible/commodity/issues/3).
+ CommodityType#from_str() now accepts &str or String for arguments using AsRef\<str\> and Into\<String\> to address [#2](https://github.com/kellpossible/commodity/issues/2).
+ CommodityType#from_currency_alpha3() now accepts &str or String using AsRef\<str\> to address [#2](https://github.com/kellpossible/commodity/issues/2).

## v0.3.0

+ Renamed `Currency` to `CommodityType` and associated member variables to address issue #1.
+ Renamed `CurrencyCode` to `CommodityTypeID` and associated member variables.
+ Renamed errors associated with `CommodityType` and `CommodityTypeID`.

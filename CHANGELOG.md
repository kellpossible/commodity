# Changelog for Commodity

## v0.3.1

+ Bump `rust_decimal` dependency up to using generic version `1` to address [#5](https://github.com/kellpossible/doublecount/issues/5).
+ CommodityType#from_str() now accepts &str or String for arguments using AsRef\<str\> and Into\<String\> to address [#4](https://github.com/kellpossible/doublecount/issues/4).
+ CommodityType#from_currency_alpha3() now accepts &str or String using AsRef\<str\> to address [#4](https://github.com/kellpossible/doublecount/issues/4).

## v0.3.0

+ Renamed `Currency` to `CommodityType` and associated member variables to address issue #1.
+ Renamed `CurrencyCode` to `CommodityTypeID` and associated member variables.
+ Renamed errors associated with `CommodityType` and `CommodityTypeID`.

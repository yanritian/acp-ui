//! Currency model definitions

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Currency entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    /// Currency code (ISO 4217)
    pub code: String,

    /// Currency name
    pub name: String,

    /// Currency symbol
    pub symbol: String,

    /// Exchange rate relative to base currency (CNY)
    pub exchange_rate: Decimal,

    /// Decimal places for this currency
    pub decimal_places: u8,
}

impl Currency {
    /// Create a new currency
    pub fn new(code: String, name: String, symbol: String, exchange_rate: Decimal, decimal_places: u8) -> Self {
        Self {
            code,
            name,
            symbol,
            exchange_rate,
            decimal_places,
        }
    }

    /// Convert amount from this currency to base currency
    pub fn to_base(&self, amount: Decimal) -> Decimal {
        amount * self.exchange_rate
    }

    /// Convert amount from base currency to this currency
    pub fn from_base(&self, amount: Decimal) -> Decimal {
        amount / self.exchange_rate
    }

    /// Format amount with symbol
    pub fn format(&self, amount: Decimal) -> String {
        format!("{}{}", self.symbol, amount.round_dp(self.decimal_places as u32))
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self::new(
            "CNY".to_string(),
            "Chinese Yuan".to_string(),
            "¥".to_string(),
            Decimal::ONE,
            2,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_currency_conversion() {
        let usd = Currency::new(
            "USD".to_string(),
            "US Dollar".to_string(),
            "$".to_string(),
            dec!(7.2), // 1 USD = 7.2 CNY
            2,
        );

        // Convert USD to CNY
        let cny_amount = usd.to_base(dec!(100));
        assert_eq!(cny_amount, dec!(720));

        // Convert CNY to USD
        let usd_amount = usd.from_base(dec!(720));
        assert_eq!(usd_amount.round_dp(2), dec!(100));
    }

    #[test]
    fn test_currency_format() {
        let cny = Currency::default();
        assert_eq!(cny.format(dec!(1234.567)), "¥1234.57");
    }
}
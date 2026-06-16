//! Currency service implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{FinanceError, FinanceResult};
use super::model::Currency;

/// Currency service trait
#[async_trait]
pub trait CurrencyService: Send + Sync {
    /// Get currency by code
    async fn get_currency(&self, code: &str) -> FinanceResult<Currency>;

    /// List all currencies
    async fn list_all(&self) -> FinanceResult<Vec<Currency>>;

    /// Convert amount between currencies
    async fn convert(&self, amount: Decimal, from: &str, to: &str) -> FinanceResult<Decimal>;

    /// Get base currency
    async fn base_currency(&self) -> Currency;
}

/// Currency service implementation with in-memory storage
pub struct CurrencyServiceImpl {
    currencies: Arc<RwLock<HashMap<String, Currency>>>,
    base_code: String,
}

impl CurrencyServiceImpl {
    pub fn new() -> Self {
        let _currencies: HashMap<String, Currency> = HashMap::new();

        // Seed common currencies
        let seed = [
            ("CNY", "Chinese Yuan", "¥", Decimal::ONE, 2),
            ("USD", "US Dollar", "$", Decimal::new(72, 1), 2), // 7.2
            ("EUR", "Euro", "€", Decimal::new(78, 1), 2), // 7.8
            ("GBP", "British Pound", "£", Decimal::new(91, 1), 2), // 9.1
            ("JPY", "Japanese Yen", "¥", Decimal::new(48, 3), 0), // 0.048
            ("HKD", "Hong Kong Dollar", "HK$", Decimal::new(92, 2), 2), // 0.92
            ("KRW", "South Korean Won", "₩", Decimal::new(54, 4), 0), // 0.0054
        ];

        // Build HashMap
        let currencies: HashMap<String, Currency> = seed
            .into_iter()
            .map(|(code, name, symbol, rate, decimals)| {
                (
                    code.to_string(),
                    Currency::new(code.to_string(), name.to_string(), symbol.to_string(), rate, decimals),
                )
            })
            .collect();

        Self {
            currencies: Arc::new(RwLock::new(currencies)),
            base_code: "CNY".to_string(),
        }
    }
}

impl Default for CurrencyServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CurrencyService for CurrencyServiceImpl {
    async fn get_currency(&self, code: &str) -> FinanceResult<Currency> {
        let currencies = self.currencies.read().await;
        currencies
            .get(code)
            .cloned()
            .ok_or_else(|| FinanceError::CurrencyNotFound(code.to_string()))
    }

    async fn list_all(&self) -> FinanceResult<Vec<Currency>> {
        let currencies = self.currencies.read().await;
        let mut result: Vec<Currency> = currencies.values().cloned().collect();
        result.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(result)
    }

    async fn convert(&self, amount: Decimal, from: &str, to: &str) -> FinanceResult<Decimal> {
        let currencies = self.currencies.read().await;

        let from_currency = currencies
            .get(from)
            .ok_or_else(|| FinanceError::CurrencyNotFound(from.to_string()))?;

        let to_currency = currencies
            .get(to)
            .ok_or_else(|| FinanceError::CurrencyNotFound(to.to_string()))?;

        // Convert to base currency first, then to target
        let base_amount = amount * from_currency.exchange_rate;
        let target_amount = base_amount / to_currency.exchange_rate;

        Ok(target_amount.round_dp(to_currency.decimal_places as u32))
    }

    async fn base_currency(&self) -> Currency {
        self.get_currency(&self.base_code).await.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_get_currency() {
        let service = CurrencyServiceImpl::new();

        let usd = service.get_currency("USD").await.unwrap();
        assert_eq!(usd.code, "USD");
        assert_eq!(usd.symbol, "$");
    }

    #[tokio::test]
    async fn test_list_all() {
        let service = CurrencyServiceImpl::new();

        let all = service.list_all().await.unwrap();
        assert!(all.len() >= 7);
    }

    #[tokio::test]
    async fn test_currency_conversion() {
        let service = CurrencyServiceImpl::new();

        // USD to CNY
        let cny = service.convert(dec!(100), "USD", "CNY").await.unwrap();
        assert_eq!(cny, dec!(720));

        // USD to EUR
        let eur = service.convert(dec!(100), "USD", "EUR").await.unwrap();
        // 100 USD = 720 CNY = 720/7.8 EUR ≈ 92.31
        assert!(eur > dec!(90) && eur < dec!(95));
    }

    #[tokio::test]
    async fn test_invalid_currency() {
        let service = CurrencyServiceImpl::new();

        let result = service.get_currency("XYZ").await;
        assert!(result.is_err());
    }
}
//! Balance sheet model (资产负债表)
//! Accounting equation: Assets = Liabilities + Equity

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Account balance item for balance sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Account ID
    pub account_id: i64,

    /// Account name
    pub name: String,

    /// Account type
    pub account_type: String,

    /// Balance amount
    pub balance: Decimal,

    /// Currency
    pub currency: String,
}

/// Balance sheet (资产负债表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheet {
    /// Report date
    pub as_of_date: NaiveDate,

    /// Asset accounts and balances
    pub assets: Vec<AccountBalance>,

    /// Liability accounts and balances
    pub liabilities: Vec<AccountBalance>,

    /// Equity accounts and balances
    pub equity: Vec<AccountBalance>,

    /// Total assets
    pub total_assets: Decimal,

    /// Total liabilities
    pub total_liabilities: Decimal,

    /// Total equity
    pub total_equity: Decimal,

    /// Currency
    pub currency: String,

    /// Whether the balance sheet is balanced (Assets = Liabilities + Equity)
    pub is_balanced: bool,

    /// Imbalance amount (if not balanced)
    pub imbalance: Decimal,
}

impl BalanceSheet {
    /// Create a new balance sheet
    pub fn new(as_of_date: NaiveDate, currency: String) -> Self {
        Self {
            as_of_date,
            assets: Vec::new(),
            liabilities: Vec::new(),
            equity: Vec::new(),
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            currency,
            is_balanced: true,
            imbalance: Decimal::ZERO,
        }
    }

    /// Add an asset
    pub fn add_asset(mut self, account: AccountBalance) -> Self {
        self.assets.push(account.clone());
        self.total_assets += account.balance;
        self.check_balance();
        self
    }

    /// Add a liability
    pub fn add_liability(mut self, account: AccountBalance) -> Self {
        self.liabilities.push(account.clone());
        self.total_liabilities += account.balance;
        self.check_balance();
        self
    }

    /// Add equity
    pub fn add_equity(mut self, account: AccountBalance) -> Self {
        self.equity.push(account.clone());
        self.total_equity += account.balance;
        self.check_balance();
        self
    }

    /// Check if balanced
    fn check_balance(&mut self) {
        let expected_equity = self.total_assets - self.total_liabilities;
        self.imbalance = self.total_equity - expected_equity;
        self.is_balanced = self.imbalance.is_zero();
    }

    /// Calculate net assets (Assets - Liabilities)
    pub fn net_assets(&self) -> Decimal {
        self.total_assets - self.total_liabilities
    }

    /// Get balance sheet summary
    pub fn summary(&self) -> String {
        format!(
            "资产负债表 ({})\n\
            资产总计: {} {}\n\
            负债总计: {} {}\n\
            所有者权益: {} {}\n\
            净资产: {} {}\n\
            平衡状态: {}",
            self.as_of_date,
            self.total_assets, self.currency,
            self.total_liabilities, self.currency,
            self.total_equity, self.currency,
            self.net_assets(), self.currency,
            if self.is_balanced { "平衡 ✓" } else { "不平衡 ✗" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_balanced_sheet() {
        let sheet = BalanceSheet::new(
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            "CNY".to_string(),
        )
        .add_asset(AccountBalance {
            account_id: 1,
            name: "银行存款".to_string(),
            account_type: "bank".to_string(),
            balance: dec!(10000),
            currency: "CNY".to_string(),
        })
        .add_liability(AccountBalance {
            account_id: 2,
            name: "应付账款".to_string(),
            account_type: "liability".to_string(),
            balance: dec!(3000),
            currency: "CNY".to_string(),
        })
        .add_equity(AccountBalance {
            account_id: 3,
            name: "所有者权益".to_string(),
            account_type: "equity".to_string(),
            balance: dec!(7000),
            currency: "CNY".to_string(),
        });

        assert!(sheet.is_balanced);
        assert_eq!(sheet.total_assets, dec!(10000));
        assert_eq!(sheet.total_liabilities, dec!(3000));
        assert_eq!(sheet.total_equity, dec!(7000));
        assert_eq!(sheet.net_assets(), dec!(7000));
    }

    #[test]
    fn test_imbalanced_sheet() {
        let sheet = BalanceSheet::new(
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            "CNY".to_string(),
        )
        .add_asset(AccountBalance {
            account_id: 1,
            name: "银行存款".to_string(),
            account_type: "bank".to_string(),
            balance: dec!(10000),
            currency: "CNY".to_string(),
        })
        .add_equity(AccountBalance {
            account_id: 3,
            name: "所有者权益".to_string(),
            account_type: "equity".to_string(),
            balance: dec!(8000), // Should be 10000 for balance
            currency: "CNY".to_string(),
        });

        assert!(!sheet.is_balanced);
        assert_eq!(sheet.imbalance, dec!(-2000)); // Equity - Net Assets = 8000 - 10000 = -2000
    }
}
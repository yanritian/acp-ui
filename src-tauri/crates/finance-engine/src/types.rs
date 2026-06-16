//! Core type definitions for the finance engine

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export from error module
pub use crate::error::{FinanceError, FinanceResult};

// ============================================================================
// Account Types
// ============================================================================

/// Account type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    /// Bank accounts (checking, savings)
    Bank,
    /// Cash on hand
    Cash,
    /// Credit cards and lines of credit
    Credit,
    /// Assets (property, investments, inventory)
    Asset,
    /// Liabilities (loans, payables)
    Liability,
    /// Equity (owner's capital, retained earnings)
    Equity,
    /// Income accounts
    Income,
    /// Expense accounts
    Expense,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountType::Bank => write!(f, "bank"),
            AccountType::Cash => write!(f, "cash"),
            AccountType::Credit => write!(f, "credit"),
            AccountType::Asset => write!(f, "asset"),
            AccountType::Liability => write!(f, "liability"),
            AccountType::Equity => write!(f, "equity"),
            AccountType::Income => write!(f, "income"),
            AccountType::Expense => write!(f, "expense"),
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = FinanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bank" => Ok(AccountType::Bank),
            "cash" => Ok(AccountType::Cash),
            "credit" => Ok(AccountType::Credit),
            "asset" => Ok(AccountType::Asset),
            "liability" => Ok(AccountType::Liability),
            "equity" => Ok(AccountType::Equity),
            "income" => Ok(AccountType::Income),
            "expense" => Ok(AccountType::Expense),
            _ => Err(FinanceError::InvalidAccountType(s.to_string())),
        }
    }
}

/// Account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Inactive,
    Closed,
}

impl fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "active"),
            AccountStatus::Inactive => write!(f, "inactive"),
            AccountStatus::Closed => write!(f, "closed"),
        }
    }
}

// ============================================================================
// Transaction Types
// ============================================================================

/// Transaction type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Income transaction
    Income,
    /// Expense transaction
    Expense,
    /// Transfer between accounts
    Transfer,
    /// Adjustment entry
    Adjustment,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Income => write!(f, "income"),
            TransactionType::Expense => write!(f, "expense"),
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::Adjustment => write!(f, "adjustment"),
        }
    }
}

impl std::str::FromStr for TransactionType {
    type Err = FinanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "income" => Ok(TransactionType::Income),
            "expense" => Ok(TransactionType::Expense),
            "transfer" => Ok(TransactionType::Transfer),
            "adjustment" => Ok(TransactionType::Adjustment),
            _ => Err(FinanceError::InvalidTransactionType(s.to_string())),
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Cancelled,
    Reconciled,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Completed => write!(f, "completed"),
            TransactionStatus::Cancelled => write!(f, "cancelled"),
            TransactionStatus::Reconciled => write!(f, "reconciled"),
        }
    }
}

// ============================================================================
// Category Types
// ============================================================================

/// Category type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Income,
    Expense,
}

impl fmt::Display for CategoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CategoryType::Income => write!(f, "income"),
            CategoryType::Expense => write!(f, "expense"),
        }
    }
}

impl std::str::FromStr for CategoryType {
    type Err = FinanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "income" => Ok(CategoryType::Income),
            "expense" => Ok(CategoryType::Expense),
            _ => Err(FinanceError::ValidationError(format!("Invalid category type: {}", s))),
        }
    }
}

// ============================================================================
// Budget Types
// ============================================================================

/// Budget period type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BudgetPeriod {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl fmt::Display for BudgetPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetPeriod::Weekly => write!(f, "weekly"),
            BudgetPeriod::Monthly => write!(f, "monthly"),
            BudgetPeriod::Quarterly => write!(f, "quarterly"),
            BudgetPeriod::Yearly => write!(f, "yearly"),
        }
    }
}

/// Budget status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BudgetStatus {
    Active,
    Completed,
    Cancelled,
    Expired,
}

impl fmt::Display for BudgetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetStatus::Active => write!(f, "active"),
            BudgetStatus::Completed => write!(f, "completed"),
            BudgetStatus::Cancelled => write!(f, "cancelled"),
            BudgetStatus::Expired => write!(f, "expired"),
        }
    }
}

// ============================================================================
// Query and Filter Types
// ============================================================================

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> FinanceResult<Self> {
        if start > end {
            return Err(FinanceError::InvalidDateRange {
                start: start.to_string(),
                end: end.to_string(),
            });
        }
        Ok(Self { start, end })
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl Pagination {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Money amount wrapper with currency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String,
}

impl Money {
    pub fn new(amount: Decimal, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
        }
    }

    pub fn zero(currency: &str) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency: currency.to_string(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.amount < Decimal::ZERO
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Validate that an amount is positive
pub fn validate_positive_amount(amount: Decimal, field: &str) -> FinanceResult<()> {
    if amount < Decimal::ZERO {
        return Err(FinanceError::InvalidAmount(format!(
            "{} must be positive, got {}",
            field, amount
        )));
    }
    Ok(())
}

/// Validate that an amount is non-zero
pub fn validate_non_zero_amount(amount: Decimal, field: &str) -> FinanceResult<()> {
    if amount.is_zero() {
        return Err(FinanceError::InvalidAmount(format!(
            "{} must be non-zero",
            field
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rust_decimal_macros::dec;

    #[test]
    fn test_account_type_from_str() {
        assert_eq!(AccountType::from_str("bank").unwrap(), AccountType::Bank);
        assert_eq!(AccountType::from_str("CASH").unwrap(), AccountType::Cash);
        assert!(AccountType::from_str("invalid").is_err());
    }

    #[test]
    fn test_date_range_validation() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let range = DateRange::new(start, end).unwrap();
        assert!(range.contains(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()));
    }

    #[test]
    fn test_invalid_date_range() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert!(DateRange::new(start, end).is_err());
    }

    #[test]
    fn test_money_operations() {
        let money = Money::new(dec!(100.50), "CNY");
        assert!(money.is_positive());
        assert!(!money.is_zero());

        let zero = Money::zero("USD");
        assert!(zero.is_zero());
        assert!(!zero.is_positive());
    }

    #[test]
    fn test_pagination_offset() {
        let p1 = Pagination { page: 1, per_page: 20 };
        assert_eq!(p1.offset(), 0);

        let p2 = Pagination { page: 3, per_page: 10 };
        assert_eq!(p2.offset(), 20);
    }
}
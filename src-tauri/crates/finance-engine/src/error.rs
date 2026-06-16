//! Error types for the finance engine

use thiserror::Error;

/// Main error type for finance operations
#[derive(Error, Debug)]
pub enum FinanceError {
    #[error("Account not found: {0}")]
    AccountNotFound(i64),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(i64),

    #[error("Category not found: {0}")]
    CategoryNotFound(i64),

    #[error("Budget not found: {0}")]
    BudgetNotFound(i64),

    #[error("Currency not found: {0}")]
    CurrencyNotFound(String),

    #[error("Invalid account type: {0}")]
    InvalidAccountType(String),

    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(String),

    #[error("Double-entry imbalance: debit total {debit} != credit total {credit}")]
    DoubleEntryImbalance { debit: String, credit: String },

    #[error("Insufficient balance: account {account_id} has {balance}, required {required}")]
    InsufficientBalance {
        account_id: i64,
        balance: String,
        required: String,
    },

    #[error("Budget exceeded: {budget_name} allocated {allocated}, spent {spent}")]
    BudgetExceeded {
        budget_name: String,
        allocated: String,
        spent: String,
    },

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid date range: start {start} is after end {end}")]
    InvalidDateRange { start: String, end: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for finance operations
pub type FinanceResult<T> = Result<T, FinanceError>;

impl FinanceError {
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            FinanceError::InsufficientBalance { .. }
                | FinanceError::BudgetExceeded { .. }
                | FinanceError::ValidationError(_)
                | FinanceError::ConcurrencyError(_)
        )
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            FinanceError::AccountNotFound(_) => "ACCOUNT_NOT_FOUND",
            FinanceError::TransactionNotFound(_) => "TRANSACTION_NOT_FOUND",
            FinanceError::CategoryNotFound(_) => "CATEGORY_NOT_FOUND",
            FinanceError::BudgetNotFound(_) => "BUDGET_NOT_FOUND",
            FinanceError::CurrencyNotFound(_) => "CURRENCY_NOT_FOUND",
            FinanceError::InvalidAccountType(_) => "INVALID_ACCOUNT_TYPE",
            FinanceError::InvalidTransactionType(_) => "INVALID_TRANSACTION_TYPE",
            FinanceError::DoubleEntryImbalance { .. } => "DOUBLE_ENTRY_IMBALANCE",
            FinanceError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            FinanceError::BudgetExceeded { .. } => "BUDGET_EXCEEDED",
            FinanceError::InvalidAmount(_) => "INVALID_AMOUNT",
            FinanceError::InvalidDateRange { .. } => "INVALID_DATE_RANGE",
            FinanceError::DatabaseError(_) => "DATABASE_ERROR",
            FinanceError::SerializationError(_) => "SERIALIZATION_ERROR",
            FinanceError::IoError(_) => "IO_ERROR",
            FinanceError::ValidationError(_) => "VALIDATION_ERROR",
            FinanceError::ConcurrencyError(_) => "CONCURRENCY_ERROR",
            FinanceError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            FinanceError::Unknown(_) => "UNKNOWN_ERROR",
        }
    }
}
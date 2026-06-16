//! Finance Engine - Financial system with double-entry bookkeeping
//!
//! A comprehensive financial management system supporting:
//! - Account management (Bank, Cash, Credit, Asset, Liability)
//! - Transaction tracking with double-entry bookkeeping
//! - Financial reports (Balance Sheet, Income Statement, Cash Flow)
//! - Budget management and tracking
//! - Multi-currency support

// Module order matters for dependencies
pub mod error;
pub mod types;
pub mod models;
pub mod traits;

// Implementations (depend on traits)
pub mod account;
pub mod transaction;
pub mod category;
pub mod budget;

// Supporting modules
pub mod currency;
pub mod ledger;
pub mod report;
pub mod storage;

// Re-exports for convenience
pub use error::{FinanceError, FinanceResult};
pub use types::*;

// Re-export models
pub use models::{Account, Transaction, Category, Budget, BudgetItem};

// Re-export traits
pub use traits::{
    AccountRepository, AccountService,
    TransactionRepository, TransactionService,
    CategoryRepository,
    BudgetRepository, BudgetService,
    ReportService,
    CreateAccountRequest, UpdateAccountRequest, AccountFilter,
    CreateTransactionRequest, UpdateTransactionRequest, TransactionFilter,
    CreateCategoryRequest, UpdateCategoryRequest,
    CreateBudgetRequest, CreateBudgetItemRequest, UpdateBudgetRequest, BudgetFilter,
};

// Re-export specific implementations
pub use account::AccountServiceImpl;
pub use account::SqliteAccountRepository;
pub use transaction::TransactionServiceImpl;
pub use transaction::SqliteTransactionRepository;
pub use category::CategoryServiceImpl;
pub use category::SqliteCategoryRepository;
pub use budget::BudgetServiceImpl;
pub use budget::SqliteBudgetRepository;

// Re-export storage
pub use storage::FinanceDatabase;

// Re-export ledger components
pub use ledger::{JournalEntry, LedgerEngine, LedgerValidator};

// Re-export report components
pub use report::{BalanceSheet, IncomeStatement, CashFlowStatement, ReportGenerator};

// Re-export budget tracking
pub use budget::tracker::{BudgetExecution, BudgetTracker};

// Re-export currency
pub use currency::{Currency, CurrencyServiceImpl};
//! Trait definitions for the finance engine

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::error::FinanceResult;
use crate::types::{
    AccountType, AccountStatus, TransactionType, TransactionStatus,
    CategoryType, BudgetPeriod, BudgetStatus, Pagination, DateRange, Money,
};
use crate::models::{Account, Transaction, Category, Budget};
use crate::budget::tracker::BudgetExecution;
use crate::report::{BalanceSheet, IncomeStatement, CashFlowStatement};

// ============================================================================
// Repository Traits
// ============================================================================

/// Repository trait for account persistence
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, account: &CreateAccountRequest) -> FinanceResult<i64>;
    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Account>>;
    async fn list(&self, filter: &AccountFilter) -> FinanceResult<Vec<Account>>;
    async fn update(&self, id: i64, update: &UpdateAccountRequest) -> FinanceResult<Account>;
    async fn delete(&self, id: i64) -> FinanceResult<()>;
    async fn get_balance(&self, id: i64) -> FinanceResult<Decimal>;
}

/// Repository trait for transaction persistence
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create(&self, transaction: &CreateTransactionRequest) -> FinanceResult<i64>;
    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Transaction>>;
    async fn list(&self, filter: &TransactionFilter) -> FinanceResult<Vec<Transaction>>;
    async fn update(&self, id: i64, update: &UpdateTransactionRequest) -> FinanceResult<Transaction>;
    async fn delete(&self, id: i64) -> FinanceResult<()>;
    async fn find_by_account(&self, account_id: i64) -> FinanceResult<Vec<Transaction>>;
    async fn find_by_date_range(&self, range: &DateRange) -> FinanceResult<Vec<Transaction>>;
}

/// Repository trait for category persistence
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn create(&self, category: &CreateCategoryRequest) -> FinanceResult<i64>;
    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Category>>;
    async fn list_all(&self) -> FinanceResult<Vec<Category>>;
    async fn list_by_type(&self, category_type: CategoryType) -> FinanceResult<Vec<Category>>;
    async fn update(&self, id: i64, update: &UpdateCategoryRequest) -> FinanceResult<Category>;
    async fn delete(&self, id: i64) -> FinanceResult<()>;
}

/// Repository trait for budget persistence
#[async_trait]
pub trait BudgetRepository: Send + Sync {
    async fn create(&self, budget: &CreateBudgetRequest) -> FinanceResult<i64>;
    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Budget>>;
    async fn list(&self, filter: &BudgetFilter) -> FinanceResult<Vec<Budget>>;
    async fn update(&self, id: i64, update: &UpdateBudgetRequest) -> FinanceResult<Budget>;
    async fn delete(&self, id: i64) -> FinanceResult<()>;
    async fn find_active(&self) -> FinanceResult<Vec<Budget>>;
}

// ============================================================================
// Service Traits
// ============================================================================

/// Account service trait
#[async_trait]
pub trait AccountService: Send + Sync {
    async fn create_account(&self, request: CreateAccountRequest) -> FinanceResult<Account>;
    async fn get_account(&self, id: i64) -> FinanceResult<Account>;
    async fn list_accounts(&self, filter: AccountFilter) -> FinanceResult<Vec<Account>>;
    async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> FinanceResult<Account>;
    async fn delete_account(&self, id: i64) -> FinanceResult<()>;
    async fn get_balance(&self, id: i64) -> FinanceResult<Money>;
}

/// Transaction service trait
#[async_trait]
pub trait TransactionService: Send + Sync {
    async fn create_transaction(&self, request: CreateTransactionRequest) -> FinanceResult<Transaction>;
    async fn get_transaction(&self, id: i64) -> FinanceResult<Transaction>;
    async fn list_transactions(&self, filter: TransactionFilter) -> FinanceResult<Vec<Transaction>>;
    async fn update_transaction(&self, id: i64, request: UpdateTransactionRequest) -> FinanceResult<Transaction>;
    async fn delete_transaction(&self, id: i64) -> FinanceResult<()>;
}

/// Report service trait
#[async_trait]
pub trait ReportService: Send + Sync {
    async fn balance_sheet(&self, as_of_date: NaiveDate) -> FinanceResult<BalanceSheet>;
    async fn income_statement(&self, range: DateRange) -> FinanceResult<IncomeStatement>;
    async fn cash_flow_statement(&self, range: DateRange) -> FinanceResult<CashFlowStatement>;
}

/// Budget service trait
#[async_trait]
pub trait BudgetService: Send + Sync {
    async fn create_budget(&self, request: CreateBudgetRequest) -> FinanceResult<Budget>;
    async fn get_budget(&self, id: i64) -> FinanceResult<Budget>;
    async fn list_budgets(&self, filter: BudgetFilter) -> FinanceResult<Vec<Budget>>;
    async fn update_budget(&self, id: i64, request: UpdateBudgetRequest) -> FinanceResult<Budget>;
    async fn delete_budget(&self, id: i64) -> FinanceResult<()>;
    async fn track_execution(&self, budget_id: i64) -> FinanceResult<BudgetExecution>;
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub account_type: AccountType,
    pub currency: String,
    pub initial_balance: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub status: Option<AccountStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountFilter {
    pub account_type: Option<AccountType>,
    pub status: Option<AccountStatus>,
    pub currency: Option<String>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub date: NaiveDate,
    pub description: String,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub currency: String,
    pub from_account_id: Option<i64>,
    pub to_account_id: Option<i64>,
    pub category_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionRequest {
    pub description: Option<String>,
    pub status: Option<TransactionStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionFilter {
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub date_range: Option<DateRange>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub category_type: CategoryType,
    pub parent_id: Option<i64>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBudgetRequest {
    pub name: String,
    pub period: BudgetPeriod,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub items: Vec<CreateBudgetItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBudgetItemRequest {
    pub category_id: i64,
    pub allocated: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBudgetRequest {
    pub name: Option<String>,
    pub status: Option<BudgetStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetFilter {
    pub status: Option<BudgetStatus>,
    pub period: Option<BudgetPeriod>,
    pub pagination: Option<Pagination>,
}
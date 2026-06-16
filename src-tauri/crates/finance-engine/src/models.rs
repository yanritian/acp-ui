//! Core model definitions shared across the finance engine

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{
    AccountType, AccountStatus, TransactionType, TransactionStatus,
    CategoryType, BudgetPeriod, BudgetStatus,
};

// ============================================================================
// Account Model
// ============================================================================

/// Account entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier
    pub id: i64,

    /// Account name
    pub name: String,

    /// Account type (Bank, Cash, Credit, Asset, Liability)
    pub account_type: AccountType,

    /// Currency code (CNY, USD, EUR, etc.)
    pub currency: String,

    /// Current balance
    pub balance: Decimal,

    /// Initial balance when created
    pub initial_balance: Decimal,

    /// Account status
    pub status: AccountStatus,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Account {
    /// Create a new account with default values
    pub fn new(id: i64, name: String, account_type: AccountType, currency: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            account_type,
            currency,
            balance: Decimal::ZERO,
            initial_balance: Decimal::ZERO,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create account with initial balance
    pub fn with_initial_balance(mut self, balance: Decimal) -> Self {
        self.initial_balance = balance;
        self.balance = balance;
        self.updated_at = Utc::now();
        self
    }

    /// Check if account is active
    pub fn is_active(&self) -> bool {
        self.status == AccountStatus::Active
    }

    /// Get effective balance based on account type
    pub fn effective_balance(&self) -> Decimal {
        match self.account_type {
            AccountType::Liability | AccountType::Credit => -self.balance,
            _ => self.balance,
        }
    }
}

// ============================================================================
// Transaction Model
// ============================================================================

/// Transaction entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier
    pub id: i64,

    /// Transaction date
    pub date: NaiveDate,

    /// Description
    pub description: String,

    /// Transaction type
    pub transaction_type: TransactionType,

    /// Amount
    pub amount: Decimal,

    /// Currency code
    pub currency: String,

    /// Source account (for expense/transfer)
    pub from_account_id: Option<i64>,

    /// Destination account (for income/transfer)
    pub to_account_id: Option<i64>,

    /// Category ID
    pub category_id: Option<i64>,

    /// Reference number (optional)
    pub reference: Option<String>,

    /// Transaction status
    pub status: TransactionStatus,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Check if transaction affects balance (not pending/cancelled)
    pub fn affects_balance(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Completed | TransactionStatus::Reconciled
        )
    }
}

// ============================================================================
// Category Model
// ============================================================================

/// Category entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// Unique identifier
    pub id: i64,

    /// Category name
    pub name: String,

    /// Category type (Income or Expense)
    pub category_type: CategoryType,

    /// Parent category ID (for hierarchical categories)
    pub parent_id: Option<i64>,

    /// Icon name (optional)
    pub icon: Option<String>,

    /// Color code (optional, hex format)
    pub color: Option<String>,
}

impl Category {
    /// Create a new category
    pub fn new(id: i64, name: String, category_type: CategoryType) -> Self {
        Self {
            id,
            name,
            category_type,
            parent_id: None,
            icon: None,
            color: None,
        }
    }
}

// ============================================================================
// Budget Model
// ============================================================================

/// Budget entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Unique identifier
    pub id: i64,

    /// Budget name
    pub name: String,

    /// Budget period
    pub period: BudgetPeriod,

    /// Start date
    pub start_date: NaiveDate,

    /// End date
    pub end_date: NaiveDate,

    /// Budget items (category allocations)
    pub items: Vec<BudgetItem>,

    /// Budget status
    pub status: BudgetStatus,

    /// Total allocated amount
    pub total_allocated: Decimal,
}

impl Budget {
    /// Create a new budget
    pub fn new(id: i64, name: String, period: BudgetPeriod, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            id,
            name,
            period,
            start_date,
            end_date,
            items: Vec::new(),
            status: BudgetStatus::Active,
            total_allocated: Decimal::ZERO,
        }
    }

    /// Add a budget item
    pub fn add_item(mut self, item: BudgetItem) -> Self {
        self.items.push(item.clone());
        self.total_allocated += item.allocated;
        self
    }

    /// Calculate total spent
    pub fn total_spent(&self) -> Decimal {
        self.items.iter().map(|i| i.spent).sum()
    }

    /// Calculate remaining budget
    pub fn remaining(&self) -> Decimal {
        self.total_allocated - self.total_spent()
    }

    /// Calculate utilization rate
    pub fn utilization_rate(&self) -> f64 {
        if self.total_allocated.is_zero() {
            return 0.0;
        }
        (self.total_spent() / self.total_allocated)
            .try_into()
            .unwrap_or(0.0)
            * 100.0
    }

    /// Check if budget covers a date
    pub fn covers_date(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }
}

/// Budget item (category allocation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetItem {
    /// Unique identifier
    pub id: i64,

    /// Budget ID
    pub budget_id: i64,

    /// Category ID
    pub category_id: i64,

    /// Allocated amount
    pub allocated: Decimal,

    /// Spent amount
    pub spent: Decimal,
}

impl BudgetItem {
    /// Create a new budget item
    pub fn new(id: i64, budget_id: i64, category_id: i64, allocated: Decimal) -> Self {
        Self {
            id,
            budget_id,
            category_id,
            allocated,
            spent: Decimal::ZERO,
        }
    }
}
//! Budget module - Budget management and tracking

pub mod repository;
pub mod tracker;
mod service;
mod sqlite_repository;

// Re-export models from models.rs
pub use crate::models::{Budget, BudgetItem};
pub use crate::types::{BudgetPeriod, BudgetStatus};

pub use repository::InMemoryBudgetRepository;
pub use sqlite_repository::SqliteBudgetRepository;
pub use tracker::{BudgetTracker, BudgetExecution, BudgetItemExecution};
pub use service::BudgetServiceImpl;

// Re-export traits from crate level
pub use crate::traits::{BudgetRepository, BudgetService, CreateBudgetRequest, CreateBudgetItemRequest, UpdateBudgetRequest, BudgetFilter};
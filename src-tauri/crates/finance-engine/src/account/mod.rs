//! Account module - Account management and balance tracking

mod repository;
mod sqlite_repository;
mod service;

// Re-export models from models.rs
pub use crate::models::Account;
pub use crate::types::{AccountType, AccountStatus};

pub use repository::InMemoryAccountRepository;
pub use sqlite_repository::SqliteAccountRepository;
pub use service::AccountServiceImpl;

// Re-export traits from crate level
pub use crate::traits::{AccountRepository, AccountService, CreateAccountRequest, UpdateAccountRequest, AccountFilter};
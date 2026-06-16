//! Transaction module - Transaction management and tracking

mod repository;
mod sqlite_repository;
mod service;

// Re-export models from models.rs
pub use crate::models::Transaction;
pub use crate::types::{TransactionType, TransactionStatus};

pub use repository::InMemoryTransactionRepository;
pub use sqlite_repository::SqliteTransactionRepository;
pub use service::TransactionServiceImpl;

// Re-export traits from crate level
pub use crate::traits::{TransactionRepository, TransactionService, CreateTransactionRequest, UpdateTransactionRequest, TransactionFilter};
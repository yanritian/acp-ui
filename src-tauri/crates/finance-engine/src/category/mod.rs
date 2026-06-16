//! Category module - Transaction categorization

mod repository;
mod sqlite_repository;
mod service;

// Re-export models from models.rs
pub use crate::models::Category;
pub use crate::types::CategoryType;

pub use repository::InMemoryCategoryRepository;
pub use sqlite_repository::SqliteCategoryRepository;
pub use service::CategoryServiceImpl;

// Re-export traits from crate level
pub use crate::traits::{CategoryRepository, CreateCategoryRequest, UpdateCategoryRequest};
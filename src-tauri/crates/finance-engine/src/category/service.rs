//! Category service implementation

use async_trait::async_trait;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{CategoryRepository, CreateCategoryRequest, UpdateCategoryRequest};
use crate::types::CategoryType;
use crate::models::Category;

/// Category service trait (extension)
#[async_trait]
pub trait CategoryServiceExt: Send + Sync {
    async fn create_category(&self, request: CreateCategoryRequest) -> FinanceResult<Category>;
    async fn get_category(&self, id: i64) -> FinanceResult<Category>;
    async fn list_all_categories(&self) -> FinanceResult<Vec<Category>>;
    async fn list_income_categories(&self) -> FinanceResult<Vec<Category>>;
    async fn list_expense_categories(&self) -> FinanceResult<Vec<Category>>;
    async fn update_category(&self, id: i64, request: UpdateCategoryRequest) -> FinanceResult<Category>;
    async fn delete_category(&self, id: i64) -> FinanceResult<()>;
}

/// Category service implementation
pub struct CategoryServiceImpl<R: CategoryRepository> {
    repository: R,
}

impl<R: CategoryRepository> CategoryServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CategoryRepository + 'static> CategoryServiceExt for CategoryServiceImpl<R> {
    async fn create_category(&self, request: CreateCategoryRequest) -> FinanceResult<Category> {
        if let Some(parent_id) = request.parent_id {
            let parent = self.repository.find_by_id(parent_id).await?;
            if parent.is_none() {
                return Err(FinanceError::CategoryNotFound(parent_id));
            }
            if let Some(p) = parent {
                if p.category_type != request.category_type {
                    return Err(FinanceError::ValidationError(
                        "Parent category must have the same type".to_string(),
                    ));
                }
            }
        }

        let id = self.repository.create(&request).await?;
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::CategoryNotFound(id))
    }

    async fn get_category(&self, id: i64) -> FinanceResult<Category> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::CategoryNotFound(id))
    }

    async fn list_all_categories(&self) -> FinanceResult<Vec<Category>> {
        self.repository.list_all().await
    }

    async fn list_income_categories(&self) -> FinanceResult<Vec<Category>> {
        self.repository.list_by_type(CategoryType::Income).await
    }

    async fn list_expense_categories(&self) -> FinanceResult<Vec<Category>> {
        self.repository.list_by_type(CategoryType::Expense).await
    }

    async fn update_category(&self, id: i64, request: UpdateCategoryRequest) -> FinanceResult<Category> {
        self.get_category(id).await?;
        self.repository.update(id, &request).await
    }

    async fn delete_category(&self, id: i64) -> FinanceResult<()> {
        let all = self.repository.list_all().await?;
        let has_children = all.iter().any(|c| c.parent_id == Some(id));
        if has_children {
            return Err(FinanceError::ValidationError(
                "Cannot delete category with sub-categories".to_string(),
            ));
        }

        self.repository.delete(id).await
    }
}
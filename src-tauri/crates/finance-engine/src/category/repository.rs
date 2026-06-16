//! Category repository implementations

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{CategoryRepository, CreateCategoryRequest, UpdateCategoryRequest};
use crate::models::Category;
use crate::types::CategoryType;

/// In-memory category repository
pub struct InMemoryCategoryRepository {
    categories: Arc<RwLock<HashMap<i64, Category>>>,
    next_id: Arc<RwLock<i64>>,
}

impl InMemoryCategoryRepository {
    pub fn new() -> Self {
        Self {
            categories: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    pub fn with_seed_data() -> Self {
        let repo = Self::new();

        // Use blocking read to initialize seed data
        {
            let mut categories = repo.categories.blocking_write();
            let mut next_id = repo.next_id.blocking_write();

            let seed = [
                ("工资", CategoryType::Income, "💼", "#4CAF50"),
                ("奖金", CategoryType::Income, "🎁", "#8BC34A"),
                ("投资收益", CategoryType::Income, "📈", "#CDDC39"),
                ("餐饮", CategoryType::Expense, "🍽️", "#FF5722"),
                ("交通", CategoryType::Expense, "🚗", "#FF9800"),
                ("购物", CategoryType::Expense, "🛒", "#E91E63"),
                ("娱乐", CategoryType::Expense, "🎮", "#9C27B0"),
                ("医疗", CategoryType::Expense, "🏥", "#00BCD4"),
            ];

            for (name, type_, icon, color) in seed {
                let id = *next_id;
                *next_id += 1;

                categories.insert(id, Category {
                    id,
                    name: name.to_string(),
                    category_type: type_,
                    parent_id: None,
                    icon: Some(icon.to_string()),
                    color: Some(color.to_string()),
                });
            }
        }

        repo
    }

    async fn get_next_id(&self) -> i64 {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }
}

impl Default for InMemoryCategoryRepository {
    fn default() -> Self {
        Self::with_seed_data()
    }
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepository {
    async fn create(&self, request: &CreateCategoryRequest) -> FinanceResult<i64> {
        let id = self.get_next_id().await;

        let category = Category {
            id,
            name: request.name.clone(),
            category_type: request.category_type,
            parent_id: request.parent_id,
            icon: request.icon.clone(),
            color: request.color.clone(),
        };

        let mut categories = self.categories.write().await;
        categories.insert(id, category);

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Category>> {
        let categories = self.categories.read().await;
        Ok(categories.get(&id).cloned())
    }

    async fn list_all(&self) -> FinanceResult<Vec<Category>> {
        let categories = self.categories.read().await;
        let mut result: Vec<Category> = categories.values().cloned().collect();
        result.sort_by_key(|c| c.id);
        Ok(result)
    }

    async fn list_by_type(&self, category_type: CategoryType) -> FinanceResult<Vec<Category>> {
        let categories = self.categories.read().await;
        let mut result: Vec<Category> = categories
            .values()
            .filter(|c| c.category_type == category_type)
            .cloned()
            .collect();
        result.sort_by_key(|c| c.id);
        Ok(result)
    }

    async fn update(&self, id: i64, update: &UpdateCategoryRequest) -> FinanceResult<Category> {
        let mut categories = self.categories.write().await;

        let category = categories
            .get_mut(&id)
            .ok_or(FinanceError::CategoryNotFound(id))?;

        if let Some(name) = &update.name {
            category.name = name.clone();
        }
        if let Some(icon) = &update.icon {
            category.icon = Some(icon.clone());
        }
        if let Some(color) = &update.color {
            category.color = Some(color.clone());
        }

        Ok(category.clone())
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        let mut categories = self.categories.write().await;

        if categories.remove(&id).is_none() {
            return Err(FinanceError::CategoryNotFound(id));
        }

        Ok(())
    }
}
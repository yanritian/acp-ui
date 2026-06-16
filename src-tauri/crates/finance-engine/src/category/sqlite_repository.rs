//! SQLite category repository implementation

use async_trait::async_trait;
use rusqlite::OptionalExtension;
use std::str::FromStr;

use crate::error::{FinanceError, FinanceResult};
use crate::storage::FinanceDatabase;
use crate::traits::{CategoryRepository, CreateCategoryRequest, UpdateCategoryRequest};
use crate::types::CategoryType;
use crate::models::Category;

/// SQLite-backed category repository
pub struct SqliteCategoryRepository {
    db: FinanceDatabase,
}

impl SqliteCategoryRepository {
    pub fn new(db: FinanceDatabase) -> Self {
        Self { db }
    }

    fn parse_category_type(s: &str) -> FinanceResult<CategoryType> {
        CategoryType::from_str(s)
            .map_err(|_| FinanceError::ValidationError(format!("Invalid category type: {}", s)))
    }

    fn row_to_category(row: &rusqlite::Row) -> Result<Category, rusqlite::Error> {
        let type_str: String = row.get(2)?;
        let cat_type = Self::parse_category_type(&type_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            category_type: cat_type,
            parent_id: row.get(3)?,
            icon: row.get(4)?,
            color: row.get(5)?,
        })
    }
}

#[async_trait]
impl CategoryRepository for SqliteCategoryRepository {
    async fn create(&self, request: &CreateCategoryRequest) -> FinanceResult<i64> {
        let id = self.db.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO categories (name, category_type, parent_id, icon, color)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    &request.name,
                    request.category_type.to_string(),
                    request.parent_id,
                    &request.icon,
                    &request.color,
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })?;

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Category>> {
        self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT id, name, category_type, parent_id, icon, color FROM categories WHERE id = ?1",
                [id],
                Self::row_to_category,
            )
            .optional()
            .map_err(FinanceError::from)
        })
    }

    async fn list_all(&self) -> FinanceResult<Vec<Category>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, category_type, parent_id, icon, color FROM categories ORDER BY id"
            )?;
            let categories: Vec<Category> = stmt
                .query_map([], Self::row_to_category)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(categories)
        })
    }

    async fn list_by_type(&self, category_type: CategoryType) -> FinanceResult<Vec<Category>> {
        self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, category_type, parent_id, icon, color FROM categories WHERE category_type = ?1 ORDER BY id"
            )?;
            let categories: Vec<Category> = stmt
                .query_map([category_type.to_string()], Self::row_to_category)?
                .filter_map(|r| r.ok())
                .collect();
            Ok(categories)
        })
    }

    async fn update(&self, id: i64, update: &UpdateCategoryRequest) -> FinanceResult<Category> {
        self.db.with_transaction(|tx| {
            let exists: bool = tx.query_row(
                "SELECT COUNT(*) > 0 FROM categories WHERE id = ?1",
                [id],
                |row| row.get(0),
            )?;

            if !exists {
                return Err(FinanceError::CategoryNotFound(id));
            }

            let mut sql = String::from("UPDATE categories SET ");
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            let mut has_update = false;

            if let Some(name) = &update.name {
                sql.push_str("name = ?");
                params.push(Box::new(name.clone()));
                has_update = true;
            }

            if let Some(icon) = &update.icon {
                if has_update { sql.push_str(", "); }
                sql.push_str("icon = ?");
                params.push(Box::new(icon.clone()));
                has_update = true;
            }

            if let Some(color) = &update.color {
                if has_update { sql.push_str(", "); }
                sql.push_str("color = ?");
                params.push(Box::new(color.clone()));
                has_update = true;
            }

            if !has_update {
                // No fields to update, just return the category
                return tx.query_row(
                    "SELECT id, name, category_type, parent_id, icon, color FROM categories WHERE id = ?1",
                    [id],
                    Self::row_to_category,
                ).map_err(FinanceError::from);
            }

            sql.push_str(" WHERE id = ?");
            params.push(Box::new(id));

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            tx.execute(&sql, params_refs.as_slice())?;

            tx.query_row(
                "SELECT id, name, category_type, parent_id, icon, color FROM categories WHERE id = ?1",
                [id],
                Self::row_to_category,
            ).map_err(FinanceError::from)
        })
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        self.db.with_transaction(|tx| {
            let deleted = tx.execute("DELETE FROM categories WHERE id = ?1", [id])?;
            if deleted == 0 {
                return Err(FinanceError::CategoryNotFound(id));
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::CategoryRepository;

    async fn create_test_repo() -> SqliteCategoryRepository {
        let db = FinanceDatabase::in_memory().unwrap();
        SqliteCategoryRepository::new(db)
    }

    #[tokio::test]
    async fn test_seed_data_loaded() {
        let repo = create_test_repo().await;
        let categories = repo.list_all().await.unwrap();
        // 8 seed categories
        assert!(categories.len() >= 8);
    }

    #[tokio::test]
    async fn test_list_income_categories() {
        let repo = create_test_repo().await;
        let categories = repo.list_by_type(CategoryType::Income).await.unwrap();
        assert!(categories.len() >= 3); // 工资, 奖金, 投资收益
    }

    #[tokio::test]
    async fn test_list_expense_categories() {
        let repo = create_test_repo().await;
        let categories = repo.list_by_type(CategoryType::Expense).await.unwrap();
        assert!(categories.len() >= 5); // 餐饮, 交通, 购物, 娱乐, 医疗
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let repo = create_test_repo().await;
        // Category ID 1 is 工资
        let category = repo.find_by_id(1).await.unwrap().unwrap();
        assert_eq!(category.name, "工资");
        assert_eq!(category.category_type, CategoryType::Income);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = create_test_repo().await;
        let result = repo.find_by_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_category() {
        let repo = create_test_repo().await;

        let id = repo.create(&CreateCategoryRequest {
            name: "Test Category".to_string(),
            category_type: CategoryType::Expense,
            parent_id: None,
            icon: Some("📌".to_string()),
            color: Some("#123456".to_string()),
        }).await.unwrap();

        assert!(id > 0);
        let category = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(category.name, "Test Category");
    }

    #[tokio::test]
    async fn test_update_category() {
        let repo = create_test_repo().await;

        let updated = repo.update(1, &UpdateCategoryRequest {
            name: Some("Updated Name".to_string()),
            icon: None,
            color: None,
        }).await.unwrap();

        assert_eq!(updated.name, "Updated Name");
    }

    #[tokio::test]
    async fn test_delete_category() {
        let repo = create_test_repo().await;

        // Create a new category to delete
        let id = repo.create(&CreateCategoryRequest {
            name: "To Delete".to_string(),
            category_type: CategoryType::Expense,
            parent_id: None,
            icon: None,
            color: None,
        }).await.unwrap();

        repo.delete(id).await.unwrap();
        let result = repo.find_by_id(id).await.unwrap();
        assert!(result.is_none());
    }
}
//! SQLite budget repository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rusqlite::OptionalExtension;
use std::str::FromStr;

use crate::error::{FinanceError, FinanceResult};
use crate::storage::FinanceDatabase;
use crate::traits::{BudgetRepository, CreateBudgetRequest, UpdateBudgetRequest, BudgetFilter};
use crate::types::{BudgetPeriod, BudgetStatus};
use crate::models::{Budget, BudgetItem};

/// SQLite-backed budget repository
pub struct SqliteBudgetRepository {
    db: FinanceDatabase,
}

impl SqliteBudgetRepository {
    pub fn new(db: FinanceDatabase) -> Self {
        Self { db }
    }

    fn parse_budget_period(s: &str) -> FinanceResult<BudgetPeriod> {
        match s.to_lowercase().as_str() {
            "weekly" => Ok(BudgetPeriod::Weekly),
            "monthly" => Ok(BudgetPeriod::Monthly),
            "quarterly" => Ok(BudgetPeriod::Quarterly),
            "yearly" => Ok(BudgetPeriod::Yearly),
            _ => Err(FinanceError::ValidationError(format!("Invalid period: {}", s))),
        }
    }

    fn parse_budget_status(s: &str) -> FinanceResult<BudgetStatus> {
        match s.to_lowercase().as_str() {
            "active" => Ok(BudgetStatus::Active),
            "completed" => Ok(BudgetStatus::Completed),
            "cancelled" => Ok(BudgetStatus::Cancelled),
            "expired" => Ok(BudgetStatus::Expired),
            _ => Err(FinanceError::ValidationError(format!("Invalid status: {}", s))),
        }
    }

    fn parse_date(s: &str) -> FinanceResult<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| FinanceError::ValidationError(format!("Invalid date: {}", s)))
    }

    fn parse_datetime(s: &str) -> FinanceResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| FinanceError::ValidationError(format!("Invalid datetime: {}", s)))
    }

    fn row_to_budget_item(row: &rusqlite::Row) -> Result<BudgetItem, rusqlite::Error> {
        let allocated_str: String = row.get(3)?;
        let spent_str: String = row.get(4)?;

        let allocated = Decimal::from_str(&allocated_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let spent = Decimal::from_str(&spent_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(BudgetItem {
            id: row.get(0)?,
            budget_id: row.get(1)?,
            category_id: row.get(2)?,
            allocated,
            spent,
        })
    }

    fn row_to_budget(row: &rusqlite::Row) -> Result<Budget, rusqlite::Error> {
        let period_str: String = row.get(2)?;
        let status_str: String = row.get(5)?;
        let total_str: String = row.get(6)?;

        let period = Self::parse_budget_period(&period_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let status = Self::parse_budget_status(&status_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let total_allocated = Decimal::from_str(&total_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let start_date_str: String = row.get(3)?;
        let end_date_str: String = row.get(4)?;

        let start_date = Self::parse_date(&start_date_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let end_date = Self::parse_date(&end_date_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Budget {
            id: row.get(0)?,
            name: row.get(1)?,
            period,
            start_date,
            end_date,
            items: vec![], // Items loaded separately
            status,
            total_allocated,
        })
    }
}

#[async_trait]
impl BudgetRepository for SqliteBudgetRepository {
    async fn create(&self, request: &CreateBudgetRequest) -> FinanceResult<i64> {
        let now = Utc::now();
        let status = BudgetStatus::Active;

        self.db.with_transaction(|tx| {
            // Calculate total allocated
            let total_allocated: Decimal = request.items.iter()
                .map(|i| i.allocated)
                .sum();

            // Insert budget
            tx.execute(
                "INSERT INTO budgets (name, period, start_date, end_date, status, total_allocated, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    &request.name,
                    request.period.to_string(),
                    request.start_date.to_string(),
                    request.end_date.to_string(),
                    status.to_string(),
                    total_allocated.to_string(),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )?;

            let budget_id = tx.last_insert_rowid();

            // Insert budget items
            for item in &request.items {
                tx.execute(
                    "INSERT INTO budget_items (budget_id, category_id, allocated, spent, remaining, is_exceeded)
                     VALUES (?1, ?2, ?3, '0', ?4, 0)",
                    rusqlite::params![
                        budget_id,
                        item.category_id,
                        item.allocated.to_string(),
                        item.allocated.to_string(),
                    ],
                )?;
            }

            Ok(budget_id)
        })
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Budget>> {
        self.db.with_connection(|conn| {
            // Get budget
            let budget = conn.query_row(
                "SELECT id, name, period, start_date, end_date, status, total_allocated FROM budgets WHERE id = ?1",
                [id],
                Self::row_to_budget,
            ).optional()?;

            if let Some(mut budget) = budget {
                // Load budget items
                let mut stmt = conn.prepare(
                    "SELECT id, budget_id, category_id, allocated, spent FROM budget_items WHERE budget_id = ?1"
                )?;

                let items: Vec<BudgetItem> = stmt
                    .query_map([id], Self::row_to_budget_item)?
                    .filter_map(|r| r.ok())
                    .collect();

                budget.items = items;
                Ok(Some(budget))
            } else {
                Ok(None)
            }
        })
    }

    async fn list(&self, filter: &BudgetFilter) -> FinanceResult<Vec<Budget>> {
        self.db.with_connection(|conn| {
            let mut sql = String::from(
                "SELECT id, name, period, start_date, end_date, status, total_allocated FROM budgets WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(status) = filter.status {
                sql.push_str(" AND status = ?");
                params.push(Box::new(status.to_string()));
            }

            if let Some(period) = filter.period {
                sql.push_str(" AND period = ?");
                params.push(Box::new(period.to_string()));
            }

            sql.push_str(" ORDER BY id");

            if let Some(pagination) = &filter.pagination {
                sql.push_str(" LIMIT ? OFFSET ?");
                params.push(Box::new(pagination.per_page as i64));
                params.push(Box::new(pagination.offset() as i64));
            }

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let budgets: Vec<Budget> = stmt
                .query_map(params_refs.as_slice(), Self::row_to_budget)?
                .filter_map(|r| r.ok())
                .collect();

            Ok(budgets)
        })
    }

    async fn update(&self, id: i64, update: &UpdateBudgetRequest) -> FinanceResult<Budget> {
        self.db.with_transaction(|tx| {
            let exists: bool = tx.query_row(
                "SELECT COUNT(*) > 0 FROM budgets WHERE id = ?1",
                [id],
                |row| row.get(0),
            )?;

            if !exists {
                return Err(FinanceError::BudgetNotFound(id));
            }

            let mut sql = String::from("UPDATE budgets SET updated_at = ?1");
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            params.push(Box::new(Utc::now().to_rfc3339()));

            if let Some(name) = &update.name {
                sql.push_str(", name = ?");
                params.push(Box::new(name.clone()));
            }

            if let Some(status) = update.status {
                sql.push_str(", status = ?");
                params.push(Box::new(status.to_string()));
            }

            sql.push_str(" WHERE id = ?");
            params.push(Box::new(id));

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            tx.execute(&sql, params_refs.as_slice())?;

            // Fetch updated budget with items
            let budget = tx.query_row(
                "SELECT id, name, period, start_date, end_date, status, total_allocated FROM budgets WHERE id = ?1",
                [id],
                Self::row_to_budget,
            ).map_err(FinanceError::from)?;

            // Load items
            let mut stmt = tx.prepare(
                "SELECT id, budget_id, category_id, allocated, spent FROM budget_items WHERE budget_id = ?1"
            )?;

            let items: Vec<BudgetItem> = stmt
                .query_map([id], Self::row_to_budget_item)?
                .filter_map(|r| r.ok())
                .collect();

            Ok(Budget { items, ..budget })
        })
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        self.db.with_transaction(|tx| {
            // Delete budget items first (cascade)
            tx.execute("DELETE FROM budget_items WHERE budget_id = ?1", [id])?;

            let deleted = tx.execute("DELETE FROM budgets WHERE id = ?1", [id])?;
            if deleted == 0 {
                return Err(FinanceError::BudgetNotFound(id));
            }
            Ok(())
        })
    }

    async fn find_active(&self) -> FinanceResult<Vec<Budget>> {
        let filter = BudgetFilter {
            status: Some(BudgetStatus::Active),
            ..Default::default()
        };
        self.list(&filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{BudgetRepository, CreateBudgetItemRequest};
    use crate::types::BudgetPeriod;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    async fn create_test_repo() -> SqliteBudgetRepository {
        let db = FinanceDatabase::in_memory().unwrap();
        SqliteBudgetRepository::new(db)
    }

    fn test_date_range() -> (NaiveDate, NaiveDate) {
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        (start, end)
    }

    #[tokio::test]
    async fn test_create_budget() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        let id = repo.create(&CreateBudgetRequest {
            name: "January Budget".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![
                CreateBudgetItemRequest {
                    category_id: 4, // 餐饮
                    allocated: dec!(500),
                },
            ],
        }).await.unwrap();

        assert!(id > 0);
        let budget = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(budget.name, "January Budget");
        assert_eq!(budget.items.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = create_test_repo().await;
        let result = repo.find_by_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_budget_with_items() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        let id = repo.create(&CreateBudgetRequest {
            name: "Multi Category Budget".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![
                CreateBudgetItemRequest { category_id: 4, allocated: dec!(500) },
                CreateBudgetItemRequest { category_id: 5, allocated: dec!(300) },
                CreateBudgetItemRequest { category_id: 6, allocated: dec!(200) },
            ],
        }).await.unwrap();

        let budget = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(budget.items.len(), 3);
        assert_eq!(budget.total_allocated, dec!(1000));
    }

    #[tokio::test]
    async fn test_list_budgets() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        repo.create(&CreateBudgetRequest {
            name: "Budget 1".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        repo.create(&CreateBudgetRequest {
            name: "Budget 2".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        let budgets = repo.list(&BudgetFilter::default()).await.unwrap();
        assert_eq!(budgets.len(), 2);
    }

    #[tokio::test]
    async fn test_update_budget_status() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        let id = repo.create(&CreateBudgetRequest {
            name: "Test".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        let updated = repo.update(id, &UpdateBudgetRequest {
            name: None,
            status: Some(BudgetStatus::Completed),
        }).await.unwrap();

        assert_eq!(updated.status, BudgetStatus::Completed);
    }

    #[tokio::test]
    async fn test_delete_budget() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        let id = repo.create(&CreateBudgetRequest {
            name: "To Delete".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        repo.delete(id).await.unwrap();
        let result = repo.find_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_active_budgets() {
        let repo = create_test_repo().await;
        let (start, end) = test_date_range();

        repo.create(&CreateBudgetRequest {
            name: "Active".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        let id = repo.create(&CreateBudgetRequest {
            name: "To Cancel".to_string(),
            period: BudgetPeriod::Monthly,
            start_date: start,
            end_date: end,
            items: vec![],
        }).await.unwrap();

        repo.update(id, &UpdateBudgetRequest {
            name: None,
            status: Some(BudgetStatus::Cancelled),
        }).await.unwrap();

        let active = repo.find_active().await.unwrap();
        assert_eq!(active.len(), 1);
    }
}
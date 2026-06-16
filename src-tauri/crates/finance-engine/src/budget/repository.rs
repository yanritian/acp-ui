//! Budget repository implementations

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{BudgetRepository, CreateBudgetRequest, UpdateBudgetRequest, BudgetFilter};
use crate::models::{Budget, BudgetItem};
use crate::types::BudgetStatus;

/// In-memory budget repository
pub struct InMemoryBudgetRepository {
    budgets: Arc<RwLock<HashMap<i64, Budget>>>,
    next_id: Arc<RwLock<i64>>,
    next_item_id: Arc<RwLock<i64>>,
}

impl InMemoryBudgetRepository {
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            next_item_id: Arc::new(RwLock::new(1)),
        }
    }

    async fn get_next_id(&self) -> i64 {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }

    async fn get_next_item_id(&self) -> i64 {
        let mut next_item_id = self.next_item_id.write().await;
        let id = *next_item_id;
        *next_item_id += 1;
        id
    }
}

impl Default for InMemoryBudgetRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BudgetRepository for InMemoryBudgetRepository {
    async fn create(&self, request: &CreateBudgetRequest) -> FinanceResult<i64> {
        let id = self.get_next_id().await;

        let mut budget = Budget::new(
            id,
            request.name.clone(),
            request.period,
            request.start_date,
            request.end_date,
        );

        for item_req in &request.items {
            let item_id = self.get_next_item_id().await;
            let item = BudgetItem::new(item_id, id, item_req.category_id, item_req.allocated);
            budget = budget.add_item(item);
        }

        let mut budgets = self.budgets.write().await;
        budgets.insert(id, budget);

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Budget>> {
        let budgets = self.budgets.read().await;
        Ok(budgets.get(&id).cloned())
    }

    async fn list(&self, filter: &BudgetFilter) -> FinanceResult<Vec<Budget>> {
        let budgets = self.budgets.read().await;

        let mut result: Vec<Budget> = budgets
            .values()
            .filter(|budget| {
                if let Some(status) = filter.status {
                    if budget.status != status {
                        return false;
                    }
                }
                if let Some(period) = filter.period {
                    if budget.period != period {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        result.sort_by_key(|b| b.id);
        Ok(result)
    }

    async fn update(&self, id: i64, update: &UpdateBudgetRequest) -> FinanceResult<Budget> {
        let mut budgets = self.budgets.write().await;

        let budget = budgets
            .get_mut(&id)
            .ok_or(FinanceError::BudgetNotFound(id))?;

        if let Some(name) = &update.name {
            budget.name = name.clone();
        }
        if let Some(status) = update.status {
            budget.status = status;
        }

        Ok(budget.clone())
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        let mut budgets = self.budgets.write().await;

        if budgets.remove(&id).is_none() {
            return Err(FinanceError::BudgetNotFound(id));
        }

        Ok(())
    }

    async fn find_active(&self) -> FinanceResult<Vec<Budget>> {
        let filter = BudgetFilter {
            status: Some(BudgetStatus::Active),
            ..Default::default()
        };
        self.list(&filter).await
    }
}
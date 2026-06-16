//! Budget service implementation

use async_trait::async_trait;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{BudgetService, CreateBudgetRequest, UpdateBudgetRequest, BudgetFilter, BudgetRepository};
use crate::types::validate_positive_amount;
use crate::models::Budget;
use crate::types::BudgetStatus;
use crate::budget::tracker::BudgetExecution;

/// Budget service implementation
pub struct BudgetServiceImpl<R: BudgetRepository> {
    repository: R,
}

impl<R: BudgetRepository> BudgetServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn validate_request(&self, request: &CreateBudgetRequest) -> FinanceResult<()> {
        if request.start_date > request.end_date {
            return Err(FinanceError::InvalidDateRange {
                start: request.start_date.to_string(),
                end: request.end_date.to_string(),
            });
        }

        for item in &request.items {
            validate_positive_amount(item.allocated, "budget item allocation")?;
        }

        Ok(())
    }
}

#[async_trait]
impl<R: BudgetRepository + 'static> BudgetService for BudgetServiceImpl<R> {
    async fn create_budget(&self, request: CreateBudgetRequest) -> FinanceResult<Budget> {
        self.validate_request(&request)?;

        let id = self.repository.create(&request).await?;

        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::BudgetNotFound(id))
    }

    async fn get_budget(&self, id: i64) -> FinanceResult<Budget> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::BudgetNotFound(id))
    }

    async fn list_budgets(&self, filter: BudgetFilter) -> FinanceResult<Vec<Budget>> {
        self.repository.list(&filter).await
    }

    async fn update_budget(&self, id: i64, request: UpdateBudgetRequest) -> FinanceResult<Budget> {
        self.get_budget(id).await?;
        self.repository.update(id, &request).await
    }

    async fn delete_budget(&self, id: i64) -> FinanceResult<()> {
        let budget = self.get_budget(id).await?;
        if budget.status == BudgetStatus::Active {
            return Err(FinanceError::ValidationError(
                "Cannot delete active budget. Please cancel it first.".to_string(),
            ));
        }

        self.repository.delete(id).await
    }

    async fn track_execution(&self, budget_id: i64) -> FinanceResult<BudgetExecution> {
        let budget = self.get_budget(budget_id).await?;

        Ok(BudgetExecution {
            budget_id,
            budget_name: budget.name.clone(),
            total_allocated: budget.total_allocated,
            total_spent: budget.total_spent(),
            remaining: budget.remaining(),
            utilization_rate: budget.utilization_rate(),
            items: vec![],
            status: crate::budget::tracker::BudgetExecutionStatus::OnTrack,
        })
    }
}
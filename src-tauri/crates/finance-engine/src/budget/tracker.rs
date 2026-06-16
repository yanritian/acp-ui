//! Budget tracker - Budget execution tracking and alerts

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::Transaction;
use crate::types::TransactionType;

/// Budget execution tracking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetExecution {
    pub budget_id: i64,
    pub budget_name: String,
    pub total_allocated: Decimal,
    pub total_spent: Decimal,
    pub remaining: Decimal,
    pub utilization_rate: f64,
    pub items: Vec<BudgetItemExecution>,
    pub status: BudgetExecutionStatus,
}

/// Budget execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetExecutionStatus {
    OnTrack,
    Warning,
    Exceeded,
}

/// Budget item execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetItemExecution {
    pub category_id: i64,
    pub category_name: String,
    pub allocated: Decimal,
    pub spent: Decimal,
    pub remaining: Decimal,
    pub utilization_rate: f64,
    pub is_exceeded: bool,
}

/// Budget tracker for execution monitoring
pub struct BudgetTracker;

impl BudgetTracker {
    pub fn track_execution(
        budget: &crate::models::Budget,
        transactions: &[Transaction],
        category_names: &HashMap<i64, String>,
    ) -> BudgetExecution {
        let mut category_spent: HashMap<i64, Decimal> = HashMap::new();

        for tx in transactions {
            if budget.covers_date(tx.date) && tx.transaction_type == TransactionType::Expense {
                if let Some(cat_id) = tx.category_id {
                    let current = category_spent.get(&cat_id).copied().unwrap_or(Decimal::ZERO);
                    category_spent.insert(cat_id, current + tx.amount);
                }
            }
        }

        let mut items: Vec<BudgetItemExecution> = Vec::new();
        let mut total_spent = Decimal::ZERO;

        for budget_item in &budget.items {
            let spent = category_spent.get(&budget_item.category_id).copied().unwrap_or(Decimal::ZERO);
            total_spent += spent;

            let utilization_rate = if budget_item.allocated.is_zero() {
                0.0
            } else {
                (spent / budget_item.allocated)
                    .try_into()
                    .unwrap_or(0.0)
                    * 100.0
            };

            items.push(BudgetItemExecution {
                category_id: budget_item.category_id,
                category_name: category_names
                    .get(&budget_item.category_id)
                    .cloned()
                    .unwrap_or_else(|| "未知分类".to_string()),
                allocated: budget_item.allocated,
                spent,
                remaining: budget_item.allocated - spent,
                utilization_rate,
                is_exceeded: spent > budget_item.allocated,
            });
        }

        let utilization_rate = if budget.total_allocated.is_zero() {
            0.0
        } else {
            (total_spent / budget.total_allocated)
                .try_into()
                .unwrap_or(0.0)
                * 100.0
        };

        let status = Self::determine_status(utilization_rate, &items);

        BudgetExecution {
            budget_id: budget.id,
            budget_name: budget.name.clone(),
            total_allocated: budget.total_allocated,
            total_spent,
            remaining: budget.total_allocated - total_spent,
            utilization_rate,
            items,
            status,
        }
    }

    fn determine_status(utilization_rate: f64, items: &[BudgetItemExecution]) -> BudgetExecutionStatus {
        if items.iter().any(|i| i.is_exceeded) {
            return BudgetExecutionStatus::Exceeded;
        }

        if utilization_rate >= 80.0 {
            return BudgetExecutionStatus::Warning;
        }

        BudgetExecutionStatus::OnTrack
    }

    pub fn generate_alert(execution: &BudgetExecution) -> Option<String> {
        match execution.status {
            BudgetExecutionStatus::Exceeded => {
                Some(format!(
                    "预算超支警告！总预算 {} 已花费 {} ({:.1}%)",
                    execution.total_allocated,
                    execution.total_spent,
                    execution.utilization_rate
                ))
            }
            BudgetExecutionStatus::Warning => {
                Some(format!(
                    "预算接近上限！已使用 {:.1}% ({} / {})",
                    execution.utilization_rate,
                    execution.total_spent,
                    execution.total_allocated
                ))
            }
            BudgetExecutionStatus::OnTrack => None,
        }
    }
}
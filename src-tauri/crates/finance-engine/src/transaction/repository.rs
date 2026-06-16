//! Transaction repository implementations

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{
    TransactionRepository, CreateTransactionRequest, UpdateTransactionRequest,
    TransactionFilter,
};
use crate::types::{DateRange, TransactionStatus};
use crate::models::Transaction;

/// In-memory transaction repository for testing and development
pub struct InMemoryTransactionRepository {
    transactions: Arc<RwLock<HashMap<i64, Transaction>>>,
    next_id: Arc<RwLock<i64>>,
}

impl InMemoryTransactionRepository {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    async fn get_next_id(&self) -> i64 {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }
}

impl Default for InMemoryTransactionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransactionRepository for InMemoryTransactionRepository {
    async fn create(&self, request: &CreateTransactionRequest) -> FinanceResult<i64> {
        let id = self.get_next_id().await;
        let now = Utc::now();

        let transaction = Transaction {
            id,
            date: request.date,
            description: request.description.clone(),
            transaction_type: request.transaction_type,
            amount: request.amount,
            currency: request.currency.clone(),
            from_account_id: request.from_account_id,
            to_account_id: request.to_account_id,
            category_id: request.category_id,
            reference: None,
            status: TransactionStatus::Pending,
            created_at: now,
            updated_at: now,
        };

        let mut transactions = self.transactions.write().await;
        transactions.insert(id, transaction);

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Transaction>> {
        let transactions = self.transactions.read().await;
        Ok(transactions.get(&id).cloned())
    }

    async fn list(&self, filter: &TransactionFilter) -> FinanceResult<Vec<Transaction>> {
        let transactions = self.transactions.read().await;

        let mut result: Vec<Transaction> = transactions
            .values()
            .filter(|tx| {
                if let Some(account_id) = filter.account_id {
                    if tx.from_account_id != Some(account_id) && tx.to_account_id != Some(account_id) {
                        return false;
                    }
                }
                if let Some(category_id) = filter.category_id {
                    if tx.category_id != Some(category_id) {
                        return false;
                    }
                }
                if let Some(tx_type) = filter.transaction_type {
                    if tx.transaction_type != tx_type {
                        return false;
                    }
                }
                if let Some(status) = filter.status {
                    if tx.status != status {
                        return false;
                    }
                }
                if let Some(range) = &filter.date_range {
                    if !range.contains(tx.date) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        result.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(result)
    }

    async fn update(&self, id: i64, update: &UpdateTransactionRequest) -> FinanceResult<Transaction> {
        let mut transactions = self.transactions.write().await;

        let transaction = transactions
            .get_mut(&id)
            .ok_or(FinanceError::TransactionNotFound(id))?;

        if let Some(description) = &update.description {
            transaction.description = description.clone();
        }
        if let Some(status) = update.status {
            transaction.status = status;
        }
        transaction.updated_at = Utc::now();

        Ok(transaction.clone())
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        let mut transactions = self.transactions.write().await;

        if transactions.remove(&id).is_none() {
            return Err(FinanceError::TransactionNotFound(id));
        }

        Ok(())
    }

    async fn find_by_account(&self, account_id: i64) -> FinanceResult<Vec<Transaction>> {
        let filter = TransactionFilter {
            account_id: Some(account_id),
            ..Default::default()
        };
        self.list(&filter).await
    }

    async fn find_by_date_range(&self, range: &DateRange) -> FinanceResult<Vec<Transaction>> {
        let filter = TransactionFilter {
            date_range: Some(range.clone()),
            ..Default::default()
        };
        self.list(&filter).await
    }
}
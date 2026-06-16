//! Transaction service implementation

use async_trait::async_trait;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{
    TransactionService, CreateTransactionRequest, UpdateTransactionRequest,
    TransactionFilter, TransactionRepository,
};
use crate::types::{validate_positive_amount, TransactionStatus, TransactionType};
use crate::models::Transaction;

/// Transaction service implementation
pub struct TransactionServiceImpl<R: TransactionRepository> {
    repository: R,
}

impl<R: TransactionRepository> TransactionServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn validate_request(&self, request: &CreateTransactionRequest) -> FinanceResult<()> {
        validate_positive_amount(request.amount, "amount")?;

        match request.transaction_type {
            TransactionType::Income => {
                if request.to_account_id.is_none() {
                    return Err(FinanceError::ValidationError(
                        "Income transaction must have a destination account".to_string(),
                    ));
                }
            }
            TransactionType::Expense => {
                if request.from_account_id.is_none() {
                    return Err(FinanceError::ValidationError(
                        "Expense transaction must have a source account".to_string(),
                    ));
                }
            }
            TransactionType::Transfer => {
                if request.from_account_id.is_none() || request.to_account_id.is_none() {
                    return Err(FinanceError::ValidationError(
                        "Transfer transaction must have both source and destination accounts"
                            .to_string(),
                    ));
                }
                if request.from_account_id == request.to_account_id {
                    return Err(FinanceError::ValidationError(
                        "Source and destination accounts must be different".to_string(),
                    ));
                }
            }
            TransactionType::Adjustment => {}
        }

        Ok(())
    }
}

#[async_trait]
impl<R: TransactionRepository + 'static> TransactionService for TransactionServiceImpl<R> {
    async fn create_transaction(&self, request: CreateTransactionRequest) -> FinanceResult<Transaction> {
        self.validate_request(&request)?;

        let id = self.repository.create(&request).await?;

        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::TransactionNotFound(id))
    }

    async fn get_transaction(&self, id: i64) -> FinanceResult<Transaction> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::TransactionNotFound(id))
    }

    async fn list_transactions(&self, filter: TransactionFilter) -> FinanceResult<Vec<Transaction>> {
        self.repository.list(&filter).await
    }

    async fn update_transaction(&self, id: i64, request: UpdateTransactionRequest) -> FinanceResult<Transaction> {
        self.get_transaction(id).await?;
        self.repository.update(id, &request).await
    }

    async fn delete_transaction(&self, id: i64) -> FinanceResult<()> {
        let tx = self.get_transaction(id).await?;

        if tx.status == TransactionStatus::Completed || tx.status == TransactionStatus::Reconciled {
            return Err(FinanceError::ValidationError(
                "Cannot delete completed or reconciled transactions".to_string(),
            ));
        }

        self.repository.delete(id).await
    }
}
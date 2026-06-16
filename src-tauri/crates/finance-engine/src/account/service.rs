//! Account service implementation

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{
    AccountService, CreateAccountRequest, UpdateAccountRequest, AccountFilter, AccountRepository,
};
use crate::types::{Money, AccountStatus};
use crate::models::Account;

/// Account service implementation
pub struct AccountServiceImpl<R: AccountRepository> {
    repository: R,
}

impl<R: AccountRepository> AccountServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: AccountRepository + 'static> AccountService for AccountServiceImpl<R> {
    async fn create_account(&self, request: CreateAccountRequest) -> FinanceResult<Account> {
        if let Some(balance) = request.initial_balance {
            if balance < Decimal::ZERO {
                return Err(FinanceError::InvalidAmount(
                    "Initial balance cannot be negative".to_string(),
                ));
            }
        }

        let id = self.repository.create(&request).await?;

        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::AccountNotFound(id))
    }

    async fn get_account(&self, id: i64) -> FinanceResult<Account> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(FinanceError::AccountNotFound(id))
    }

    async fn list_accounts(&self, filter: AccountFilter) -> FinanceResult<Vec<Account>> {
        self.repository.list(&filter).await
    }

    async fn update_account(&self, id: i64, request: UpdateAccountRequest) -> FinanceResult<Account> {
        let existing = self.get_account(id).await?;

        if request.status == Some(AccountStatus::Closed) && !existing.balance.is_zero() {
            return Err(FinanceError::ValidationError(
                "Cannot close account with non-zero balance".to_string(),
            ));
        }

        self.repository.update(id, &request).await
    }

    async fn delete_account(&self, id: i64) -> FinanceResult<()> {
        let existing = self.get_account(id).await?;

        if !existing.balance.is_zero() {
            return Err(FinanceError::ValidationError(
                "Cannot delete account with non-zero balance".to_string(),
            ));
        }

        self.repository.delete(id).await
    }

    async fn get_balance(&self, id: i64) -> FinanceResult<Money> {
        let account = self.get_account(id).await?;
        Ok(Money::new(account.balance, &account.currency))
    }
}
//! Account repository implementations

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{FinanceError, FinanceResult};
use crate::traits::{
    AccountRepository, CreateAccountRequest, UpdateAccountRequest, AccountFilter,
};
use crate::types::AccountStatus;
use crate::models::Account;

/// In-memory account repository for testing and development
pub struct InMemoryAccountRepository {
    accounts: Arc<RwLock<HashMap<i64, Account>>>,
    next_id: Arc<RwLock<i64>>,
}

impl InMemoryAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
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

impl Default for InMemoryAccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn create(&self, request: &CreateAccountRequest) -> FinanceResult<i64> {
        let id = self.get_next_id().await;
        let now = Utc::now();

        let account = Account {
            id,
            name: request.name.clone(),
            account_type: request.account_type,
            currency: request.currency.clone(),
            balance: request.initial_balance.unwrap_or(rust_decimal::Decimal::ZERO),
            initial_balance: request.initial_balance.unwrap_or(rust_decimal::Decimal::ZERO),
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        };

        let mut accounts = self.accounts.write().await;
        accounts.insert(id, account);

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Account>> {
        let accounts = self.accounts.read().await;
        Ok(accounts.get(&id).cloned())
    }

    async fn list(&self, filter: &AccountFilter) -> FinanceResult<Vec<Account>> {
        let accounts = self.accounts.read().await;

        let mut result: Vec<Account> = accounts
            .values()
            .filter(|account| {
                if let Some(t) = filter.account_type {
                    if account.account_type != t {
                        return false;
                    }
                }
                if let Some(s) = filter.status {
                    if account.status != s {
                        return false;
                    }
                }
                if let Some(c) = &filter.currency {
                    if account.currency != *c {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        result.sort_by_key(|a| a.id);

        if let Some(pagination) = &filter.pagination {
            let offset = pagination.offset() as usize;
            let limit = pagination.per_page as usize;
            result = result.into_iter().skip(offset).take(limit).collect();
        }

        Ok(result)
    }

    async fn update(&self, id: i64, update: &UpdateAccountRequest) -> FinanceResult<Account> {
        let mut accounts = self.accounts.write().await;

        let account = accounts
            .get_mut(&id)
            .ok_or(FinanceError::AccountNotFound(id))?;

        if let Some(name) = &update.name {
            account.name = name.clone();
        }
        if let Some(status) = update.status {
            account.status = status;
        }
        account.updated_at = Utc::now();

        Ok(account.clone())
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        let mut accounts = self.accounts.write().await;

        if accounts.remove(&id).is_none() {
            return Err(FinanceError::AccountNotFound(id));
        }

        Ok(())
    }

    async fn get_balance(&self, id: i64) -> FinanceResult<rust_decimal::Decimal> {
        let accounts = self.accounts.read().await;

        let account = accounts
            .get(&id)
            .ok_or(FinanceError::AccountNotFound(id))?;

        Ok(account.balance)
    }
}
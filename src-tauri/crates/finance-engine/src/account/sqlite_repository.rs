//! SQLite account repository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rusqlite::OptionalExtension;
use std::str::FromStr;

use crate::error::{FinanceError, FinanceResult};
use crate::storage::FinanceDatabase;
use crate::traits::{
    AccountRepository, CreateAccountRequest, UpdateAccountRequest, AccountFilter,
};
use crate::types::{AccountStatus, AccountType};
use crate::models::Account;

/// SQLite-backed account repository
pub struct SqliteAccountRepository {
    db: FinanceDatabase,
}

impl SqliteAccountRepository {
    pub fn new(db: FinanceDatabase) -> Self {
        Self { db }
    }

    /// Parse account type from string
    fn parse_account_type(s: &str) -> FinanceResult<AccountType> {
        AccountType::from_str(s).map_err(|_| FinanceError::InvalidAccountType(s.to_string()))
    }

    /// Parse account status from string
    fn parse_account_status(s: &str) -> FinanceResult<AccountStatus> {
        match s.to_lowercase().as_str() {
            "active" => Ok(AccountStatus::Active),
            "inactive" => Ok(AccountStatus::Inactive),
            "closed" => Ok(AccountStatus::Closed),
            _ => Err(FinanceError::ValidationError(format!("Invalid status: {}", s))),
        }
    }

    /// Parse DateTime from RFC3339 string
    fn parse_datetime(s: &str) -> FinanceResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| FinanceError::ValidationError(format!("Invalid datetime: {}", s)))
    }

    /// Map account row to Account struct
    fn row_to_account(row: &rusqlite::Row) -> Result<Account, rusqlite::Error> {
        let account_type_str: String = row.get(2)?;
        let status_str: String = row.get(6)?;
        let balance_str: String = row.get(4)?;
        let initial_balance_str: String = row.get(5)?;
        let created_at_str: String = row.get(7)?;
        let updated_at_str: String = row.get(8)?;

        // Parse account type
        let account_type = Self::parse_account_type(&account_type_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Parse status
        let status = Self::parse_account_status(&status_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Parse balance
        let balance = Decimal::from_str(&balance_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Parse initial balance
        let initial_balance = Decimal::from_str(&initial_balance_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Parse datetime
        let created_at = Self::parse_datetime(&created_at_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let updated_at = Self::parse_datetime(&updated_at_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            account_type,
            currency: row.get(3)?,
            balance,
            initial_balance,
            status,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn create(&self, request: &CreateAccountRequest) -> FinanceResult<i64> {
        let now = Utc::now();
        let initial_balance = request.initial_balance.unwrap_or(Decimal::ZERO);
        let status = AccountStatus::Active;

        let id = self.db.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO accounts (name, account_type, currency, balance, initial_balance, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    &request.name,
                    request.account_type.to_string(),
                    &request.currency,
                    initial_balance.to_string(),
                    initial_balance.to_string(),
                    status.to_string(),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })?;

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Account>> {
        let result = self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, account_type, currency, balance, initial_balance, status, created_at, updated_at
                 FROM accounts WHERE id = ?1"
            )?;

            let account = stmt
                .query_row([id], Self::row_to_account)
                .optional()?;

            Ok(account)
        })?;

        Ok(result)
    }

    async fn list(&self, filter: &AccountFilter) -> FinanceResult<Vec<Account>> {
        let accounts = self.db.with_connection(|conn| {
            // Build dynamic query based on filter
            let mut sql = String::from(
                "SELECT id, name, account_type, currency, balance, initial_balance, status, created_at, updated_at
                 FROM accounts WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(t) = filter.account_type {
                sql.push_str(" AND account_type = ?");
                params.push(Box::new(t.to_string()));
            }

            if let Some(s) = filter.status {
                sql.push_str(" AND status = ?");
                params.push(Box::new(s.to_string()));
            }

            if let Some(c) = &filter.currency {
                sql.push_str(" AND currency = ?");
                params.push(Box::new(c.clone()));
            }

            sql.push_str(" ORDER BY id");

            if let Some(pagination) = &filter.pagination {
                sql.push_str(" LIMIT ? OFFSET ?");
                params.push(Box::new(pagination.per_page as i64));
                params.push(Box::new(pagination.offset() as i64));
            }

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let accounts: Vec<Account> = stmt
                .query_map(params_refs.as_slice(), Self::row_to_account)?
                .filter_map(|r| r.ok())
                .collect();

            Ok(accounts)
        })?;

        Ok(accounts)
    }

    async fn update(&self, id: i64, update: &UpdateAccountRequest) -> FinanceResult<Account> {
        let now = Utc::now();

        self.db.with_transaction(|tx| {
            // First check if account exists
            let exists: bool = tx
                .query_row("SELECT COUNT(*) > 0 FROM accounts WHERE id = ?1", [id], |row| row.get(0))?;

            if !exists {
                return Err(FinanceError::AccountNotFound(id));
            }

            // Build update query
            let mut sql = String::from("UPDATE accounts SET updated_at = ?1");
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            params.push(Box::new(now.to_rfc3339()));

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

            // Fetch updated account
            let account = tx
                .query_row(
                    "SELECT id, name, account_type, currency, balance, initial_balance, status, created_at, updated_at
                     FROM accounts WHERE id = ?1",
                    [id],
                    Self::row_to_account,
                )?;

            Ok(account)
        })
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        self.db.with_transaction(|tx| {
            let deleted = tx.execute("DELETE FROM accounts WHERE id = ?1", [id])?;
            if deleted == 0 {
                return Err(FinanceError::AccountNotFound(id));
            }
            Ok(())
        })
    }

    async fn get_balance(&self, id: i64) -> FinanceResult<Decimal> {
        self.db.with_connection(|conn| {
            let balance_str: Result<String, rusqlite::Error> = conn
                .query_row("SELECT balance FROM accounts WHERE id = ?1", [id], |row| row.get(0));

            match balance_str {
                Ok(s) => Decimal::from_str(&s)
                    .map_err(|_| FinanceError::ValidationError("Invalid balance format".to_string())),
                Err(rusqlite::Error::QueryReturnedNoRows) => Err(FinanceError::AccountNotFound(id)),
                Err(e) => Err(FinanceError::from(e)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::CreateAccountRequest;
    use rust_decimal_macros::dec;

    async fn create_test_repo() -> SqliteAccountRepository {
        let db = FinanceDatabase::in_memory().unwrap();
        SqliteAccountRepository::new(db)
    }

    #[tokio::test]
    async fn test_create_account() {
        let repo = create_test_repo().await;

        let request = CreateAccountRequest {
            name: "Test Bank".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: Some(dec!(1000)),
        };

        let id = repo.create(&request).await.unwrap();
        assert!(id > 0);

        let account = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(account.name, "Test Bank");
        assert_eq!(account.account_type, AccountType::Bank);
        assert_eq!(account.currency, "CNY");
        assert_eq!(account.balance, dec!(1000));
    }

    #[tokio::test]
    async fn test_create_account_no_initial_balance() {
        let repo = create_test_repo().await;

        let request = CreateAccountRequest {
            name: "Empty Account".to_string(),
            account_type: AccountType::Cash,
            currency: "USD".to_string(),
            initial_balance: None,
        };

        let id = repo.create(&request).await.unwrap();
        let account = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(account.balance, Decimal::ZERO);
        assert_eq!(account.initial_balance, Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = create_test_repo().await;
        let result = repo.find_by_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_all_accounts() {
        let repo = create_test_repo().await;

        // Create multiple accounts
        repo.create(&CreateAccountRequest {
            name: "Account 1".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: Some(dec!(100)),
        }).await.unwrap();

        repo.create(&CreateAccountRequest {
            name: "Account 2".to_string(),
            account_type: AccountType::Cash,
            currency: "USD".to_string(),
            initial_balance: Some(dec!(200)),
        }).await.unwrap();

        let accounts = repo.list(&AccountFilter::default()).await.unwrap();
        assert_eq!(accounts.len(), 2);
    }

    #[tokio::test]
    async fn test_list_with_type_filter() {
        let repo = create_test_repo().await;

        repo.create(&CreateAccountRequest {
            name: "Bank 1".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: None,
        }).await.unwrap();

        repo.create(&CreateAccountRequest {
            name: "Cash 1".to_string(),
            account_type: AccountType::Cash,
            currency: "CNY".to_string(),
            initial_balance: None,
        }).await.unwrap();

        let filter = AccountFilter {
            account_type: Some(AccountType::Bank),
            ..Default::default()
        };

        let accounts = repo.list(&filter).await.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_type, AccountType::Bank);
    }

    #[tokio::test]
    async fn test_list_with_pagination() {
        let repo = create_test_repo().await;

        // Create 5 accounts
        for i in 1..=5 {
            repo.create(&CreateAccountRequest {
                name: format!("Account {}", i),
                account_type: AccountType::Bank,
                currency: "CNY".to_string(),
                initial_balance: None,
            }).await.unwrap();
        }

        let filter = AccountFilter {
            pagination: Some(crate::types::Pagination { page: 1, per_page: 3 }),
            ..Default::default()
        };

        let accounts = repo.list(&filter).await.unwrap();
        assert_eq!(accounts.len(), 3);
    }

    #[tokio::test]
    async fn test_update_account_name() {
        let repo = create_test_repo().await;

        let id = repo.create(&CreateAccountRequest {
            name: "Original".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: None,
        }).await.unwrap();

        let updated = repo.update(id, &UpdateAccountRequest {
            name: Some("Updated".to_string()),
            status: None,
        }).await.unwrap();

        assert_eq!(updated.name, "Updated");
    }

    #[tokio::test]
    async fn test_update_account_status() {
        let repo = create_test_repo().await;

        let id = repo.create(&CreateAccountRequest {
            name: "Test".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: None,
        }).await.unwrap();

        let updated = repo.update(id, &UpdateAccountRequest {
            name: None,
            status: Some(AccountStatus::Inactive),
        }).await.unwrap();

        assert_eq!(updated.status, AccountStatus::Inactive);
    }

    #[tokio::test]
    async fn test_delete_account() {
        let repo = create_test_repo().await;

        let id = repo.create(&CreateAccountRequest {
            name: "To Delete".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: None,
        }).await.unwrap();

        repo.delete(id).await.unwrap();
        let result = repo.find_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let repo = create_test_repo().await;
        let result = repo.delete(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let repo = create_test_repo().await;

        let id = repo.create(&CreateAccountRequest {
            name: "Test".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: Some(dec!(500)),
        }).await.unwrap();

        let balance = repo.get_balance(id).await.unwrap();
        assert_eq!(balance, dec!(500));
    }
}
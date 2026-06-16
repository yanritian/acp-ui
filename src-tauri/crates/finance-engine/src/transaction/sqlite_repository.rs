//! SQLite transaction repository implementation

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rusqlite::OptionalExtension;
use std::str::FromStr;

use crate::error::{FinanceError, FinanceResult};
use crate::storage::FinanceDatabase;
use crate::traits::{
    TransactionRepository, CreateTransactionRequest, UpdateTransactionRequest,
    TransactionFilter,
};
use crate::types::{DateRange, TransactionStatus, TransactionType};
use crate::models::Transaction;

/// SQLite-backed transaction repository
pub struct SqliteTransactionRepository {
    db: FinanceDatabase,
}

impl SqliteTransactionRepository {
    pub fn new(db: FinanceDatabase) -> Self {
        Self { db }
    }

    /// Parse transaction type from string
    fn parse_transaction_type(s: &str) -> FinanceResult<TransactionType> {
        TransactionType::from_str(s)
            .map_err(|_| FinanceError::InvalidTransactionType(s.to_string()))
    }

    /// Parse transaction status from string
    fn parse_transaction_status(s: &str) -> FinanceResult<TransactionStatus> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TransactionStatus::Pending),
            "completed" => Ok(TransactionStatus::Completed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            "reconciled" => Ok(TransactionStatus::Reconciled),
            _ => Err(FinanceError::ValidationError(format!("Invalid status: {}", s))),
        }
    }

    /// Parse NaiveDate from string
    fn parse_date(s: &str) -> FinanceResult<NaiveDate> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| FinanceError::ValidationError(format!("Invalid date: {}", s)))
    }

    /// Map transaction row to Transaction struct
    fn row_to_transaction(row: &rusqlite::Row) -> Result<Transaction, rusqlite::Error> {
        // Columns: id(0), date(1), description(2), transaction_type(3), amount(4), currency(5),
        //          from_account_id(6), to_account_id(7), category_id(8), reference(9), status(10),
        //          created_at(11), updated_at(12)
        let tx_type_str: String = row.get(3)?;
        let amount_str: String = row.get(4)?;
        let status_str: String = row.get(10)?;

        let tx_type = Self::parse_transaction_type(&tx_type_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let status = Self::parse_transaction_status(&status_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let date_str: String = row.get(1)?;
        let date = Self::parse_date(&date_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let created_at_str: String = row.get(11)?;
        let updated_at_str: String = row.get(12)?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(Transaction {
            id: row.get(0)?,
            date,
            description: row.get(2)?,
            transaction_type: tx_type,
            amount,
            currency: row.get(5)?,
            from_account_id: row.get(6)?,
            to_account_id: row.get(7)?,
            category_id: row.get(8)?,
            reference: row.get(9)?,
            status,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl TransactionRepository for SqliteTransactionRepository {
    async fn create(&self, request: &CreateTransactionRequest) -> FinanceResult<i64> {
        let now = Utc::now();
        let status = TransactionStatus::Completed; // Default to completed

        let id = self.db.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO transactions (date, description, transaction_type, amount, currency, from_account_id, to_account_id, category_id, reference, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                rusqlite::params![
                    request.date.to_string(),
                    &request.description,
                    request.transaction_type.to_string(),
                    request.amount.to_string(),
                    &request.currency,
                    request.from_account_id,
                    request.to_account_id,
                    request.category_id,
                    None::<String>, // reference
                    status.to_string(),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )?;
            Ok(tx.last_insert_rowid())
        })?;

        Ok(id)
    }

    async fn find_by_id(&self, id: i64) -> FinanceResult<Option<Transaction>> {
        let result = self.db.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, date, description, transaction_type, amount, currency, from_account_id, to_account_id, category_id, reference, status, created_at, updated_at
                 FROM transactions WHERE id = ?1"
            )?;

            let account = stmt
                .query_row([id], Self::row_to_transaction)
                .optional()
                .map_err(FinanceError::from)?;

            Ok(account)
        })?;

        Ok(result)
    }

    async fn list(&self, filter: &TransactionFilter) -> FinanceResult<Vec<Transaction>> {
        let transactions = self.db.with_connection(|conn| {
            let mut sql = String::from(
                "SELECT id, date, description, transaction_type, amount, currency, from_account_id, to_account_id, category_id, reference, status, created_at, updated_at
                 FROM transactions WHERE 1=1"
            );
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(account_id) = filter.account_id {
                sql.push_str(" AND (from_account_id = ? OR to_account_id = ?)");
                params.push(Box::new(account_id));
                params.push(Box::new(account_id));
            }

            if let Some(category_id) = filter.category_id {
                sql.push_str(" AND category_id = ?");
                params.push(Box::new(category_id));
            }

            if let Some(tx_type) = filter.transaction_type {
                sql.push_str(" AND transaction_type = ?");
                params.push(Box::new(tx_type.to_string()));
            }

            if let Some(status) = filter.status {
                sql.push_str(" AND status = ?");
                params.push(Box::new(status.to_string()));
            }

            if let Some(range) = &filter.date_range {
                sql.push_str(" AND date >= ? AND date <= ?");
                params.push(Box::new(range.start.to_string()));
                params.push(Box::new(range.end.to_string()));
            }

            sql.push_str(" ORDER BY date DESC, id DESC");

            if let Some(pagination) = &filter.pagination {
                sql.push_str(" LIMIT ? OFFSET ?");
                params.push(Box::new(pagination.per_page as i64));
                params.push(Box::new(pagination.offset() as i64));
            }

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let transactions: Vec<Transaction> = stmt
                .query_map(params_refs.as_slice(), Self::row_to_transaction)?
                .filter_map(|r| r.ok())
                .collect();

            Ok(transactions)
        })?;

        Ok(transactions)
    }

    async fn update(&self, id: i64, update: &UpdateTransactionRequest) -> FinanceResult<Transaction> {
        let now = Utc::now();

        self.db.with_transaction(|tx| {
            let exists: bool = tx
                .query_row("SELECT COUNT(*) > 0 FROM transactions WHERE id = ?1", [id], |row| row.get(0))?;

            if !exists {
                return Err(FinanceError::TransactionNotFound(id));
            }

            let mut sql = String::from("UPDATE transactions SET updated_at = ?1");
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
            params.push(Box::new(now.to_rfc3339()));

            if let Some(description) = &update.description {
                sql.push_str(", description = ?");
                params.push(Box::new(description.clone()));
            }

            if let Some(status) = update.status {
                sql.push_str(", status = ?");
                params.push(Box::new(status.to_string()));
            }

            sql.push_str(" WHERE id = ?");
            params.push(Box::new(id));

            let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
            tx.execute(&sql, params_refs.as_slice())?;

            let tx_result = tx.query_row(
                "SELECT id, date, description, transaction_type, amount, currency, from_account_id, to_account_id, category_id, reference, status, created_at, updated_at
                 FROM transactions WHERE id = ?1",
                [id],
                Self::row_to_transaction,
            ).map_err(FinanceError::from)?;

            Ok(tx_result)
        })
    }

    async fn delete(&self, id: i64) -> FinanceResult<()> {
        self.db.with_transaction(|tx| {
            let deleted = tx.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
            if deleted == 0 {
                return Err(FinanceError::TransactionNotFound(id));
            }
            Ok(())
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{CreateTransactionRequest, CreateAccountRequest, AccountRepository};
    use crate::account::SqliteAccountRepository;
    use crate::types::{TransactionType, AccountType};
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    async fn create_test_repo_with_accounts() -> (SqliteTransactionRepository, Vec<i64>) {
        let db = FinanceDatabase::in_memory().unwrap();

        // Create accounts first (needed for foreign key constraints)
        let account_repo = SqliteAccountRepository::new(db.clone());

        let bank_id = account_repo.create(&CreateAccountRequest {
            name: "Bank".to_string(),
            account_type: AccountType::Bank,
            currency: "CNY".to_string(),
            initial_balance: Some(dec!(10000)),
        }).await.unwrap();

        let cash_id = account_repo.create(&CreateAccountRequest {
            name: "Cash".to_string(),
            account_type: AccountType::Cash,
            currency: "CNY".to_string(),
            initial_balance: Some(dec!(500)),
        }).await.unwrap();

        let tx_repo = SqliteTransactionRepository::new(db);
        (tx_repo, vec![bank_id, cash_id])
    }

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    }

    #[tokio::test]
    async fn test_create_income_transaction() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let request = CreateTransactionRequest {
            date: test_date(),
            description: "Salary".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(5000),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1), // 工资 from seed data
        };

        let id = repo.create(&request).await.unwrap();
        assert!(id > 0);

        let tx = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(tx.description, "Salary");
        assert_eq!(tx.transaction_type, TransactionType::Income);
        assert_eq!(tx.amount, dec!(5000));
    }

    #[tokio::test]
    async fn test_create_expense_transaction() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let request = CreateTransactionRequest {
            date: test_date(),
            description: "Lunch".to_string(),
            transaction_type: TransactionType::Expense,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: Some(accounts[0]),
            to_account_id: None,
            category_id: Some(4), // 餐饮 from seed data
        };

        let id = repo.create(&request).await.unwrap();
        let tx = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Expense);
        assert_eq!(tx.from_account_id, Some(accounts[0]));
    }

    #[tokio::test]
    async fn test_create_transfer_transaction() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let request = CreateTransactionRequest {
            date: test_date(),
            description: "Bank to Cash".to_string(),
            transaction_type: TransactionType::Transfer,
            amount: dec!(500),
            currency: "CNY".to_string(),
            from_account_id: Some(accounts[0]),
            to_account_id: Some(accounts[1]),
            category_id: None,
        };

        let id = repo.create(&request).await.unwrap();
        let tx = repo.find_by_id(id).await.unwrap().unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.from_account_id, Some(accounts[0]));
        assert_eq!(tx.to_account_id, Some(accounts[1]));
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let (repo, _) = create_test_repo_with_accounts().await;
        let result = repo.find_by_id(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_all_transactions() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "TX1".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "TX2".to_string(),
            transaction_type: TransactionType::Expense,
            amount: dec!(50),
            currency: "CNY".to_string(),
            from_account_id: Some(accounts[0]),
            to_account_id: None,
            category_id: Some(4),
        }).await.unwrap();

        let txs = repo.list(&TransactionFilter::default()).await.unwrap();
        assert_eq!(txs.len(), 2);
    }

    #[tokio::test]
    async fn test_list_with_account_filter() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "To Account 0".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "To Account 1".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(200),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[1]),
            category_id: Some(1),
        }).await.unwrap();

        let filter = TransactionFilter {
            account_id: Some(accounts[0]),
            ..Default::default()
        };

        let txs = repo.list(&filter).await.unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].to_account_id, Some(accounts[0]));
    }

    #[tokio::test]
    async fn test_list_with_type_filter() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "Income".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "Expense".to_string(),
            transaction_type: TransactionType::Expense,
            amount: dec!(50),
            currency: "CNY".to_string(),
            from_account_id: Some(accounts[0]),
            to_account_id: None,
            category_id: Some(4),
        }).await.unwrap();

        let filter = TransactionFilter {
            transaction_type: Some(TransactionType::Expense),
            ..Default::default()
        };

        let txs = repo.list(&filter).await.unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].transaction_type, TransactionType::Expense);
    }

    #[tokio::test]
    async fn test_list_with_date_range() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let jan1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let jan15 = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let feb1 = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();

        repo.create(&CreateTransactionRequest {
            date: jan1,
            description: "Jan 1".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.create(&CreateTransactionRequest {
            date: feb1,
            description: "Feb 1".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(200),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        let range = DateRange::new(jan1, jan15).unwrap();
        let filter = TransactionFilter {
            date_range: Some(range),
            ..Default::default()
        };

        let txs = repo.list(&filter).await.unwrap();
        assert_eq!(txs.len(), 1);
    }

    #[tokio::test]
    async fn test_update_transaction_status() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let id = repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "Test".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        let updated = repo.update(id, &UpdateTransactionRequest {
            description: None,
            status: Some(TransactionStatus::Reconciled),
        }).await.unwrap();

        assert_eq!(updated.status, TransactionStatus::Reconciled);
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        let id = repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "To Delete".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.delete(id).await.unwrap();
        let result = repo.find_by_id(id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_by_account() {
        let (repo, accounts) = create_test_repo_with_accounts().await;

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "Income to 0".to_string(),
            transaction_type: TransactionType::Income,
            amount: dec!(100),
            currency: "CNY".to_string(),
            from_account_id: None,
            to_account_id: Some(accounts[0]),
            category_id: Some(1),
        }).await.unwrap();

        repo.create(&CreateTransactionRequest {
            date: test_date(),
            description: "Expense from 1".to_string(),
            transaction_type: TransactionType::Expense,
            amount: dec!(50),
            currency: "CNY".to_string(),
            from_account_id: Some(accounts[1]),
            to_account_id: None,
            category_id: Some(4),
        }).await.unwrap();

        let txs = repo.find_by_account(accounts[0]).await.unwrap();
        assert_eq!(txs.len(), 1);
    }
}
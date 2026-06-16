//! SQLite database connection pool for finance engine
//!
//! Provides thread-safe database access with WAL mode and proper initialization.

use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;
use rusqlite::Connection;

use crate::error::{FinanceError, FinanceResult};
use crate::storage::schema::DatabaseSchema;

/// SQLite database wrapper with thread-safe access
pub struct FinanceDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl FinanceDatabase {
    /// Create a new database at the specified path
    ///
    /// Initializes tables and seed data if not present.
    pub fn new(path: &str) -> FinanceResult<Self> {
        let conn = Self::open_connection(path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;"
        )?;

        // Create tables
        for sql in DatabaseSchema::create_tables_sql() {
            conn.execute_batch(&sql)?;
        }

        // Insert seed data (ignore conflicts for existing data)
        for sql in DatabaseSchema::seed_data_sql() {
            // Use INSERT OR IGNORE to avoid duplicate key errors
            let ignore_sql = sql.replace("INSERT INTO", "INSERT OR IGNORE INTO");
            conn.execute_batch(&ignore_sql)?;
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory database for testing
    pub fn in_memory() -> FinanceResult<Self> {
        Self::new(":memory:")
    }

    /// Open a connection to the database
    fn open_connection(path: &str) -> FinanceResult<Connection> {
        if path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(Path::new(path))
        }
        .map_err(FinanceError::from)
    }

    /// Get a locked connection for database operations
    pub fn conn(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }

    /// Execute a transaction with the connection
    pub fn with_connection<F, T>(&self, f: F) -> FinanceResult<T>
    where
        F: FnOnce(&Connection) -> FinanceResult<T>,
    {
        let conn = self.conn.lock();
        f(&conn)
    }

    /// Execute a write operation in a transaction
    pub fn with_transaction<F, T>(&self, f: F) -> FinanceResult<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> FinanceResult<T>,
    {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

impl Clone for FinanceDatabase {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_database_creation() {
        let db = FinanceDatabase::in_memory().unwrap();
        // Verify database is usable by querying a table
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM currencies", [], |row| row.get(0))?)
            })
            .unwrap();
        assert!(count >= 0); // Database is initialized
    }

    #[test]
    fn test_tables_created() {
        let db = FinanceDatabase::in_memory().unwrap();

        // Check accounts table exists
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(count, 1);

        // Check transactions table exists
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transactions'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(count, 1);

        // Check all 7 tables exist
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('accounts', 'transactions', 'journal_entries', 'categories', 'budgets', 'budget_items', 'currencies')",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(count, 7);
    }

    #[test]
    fn test_seed_data_loaded() {
        let db = FinanceDatabase::in_memory().unwrap();

        // Check categories seed data (8 default categories)
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(count, 8);

        // Check currencies seed data (6 currencies: CNY, USD, EUR, GBP, JPY, HKD)
        let count: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM currencies", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn test_wal_mode_enabled() {
        let db = FinanceDatabase::in_memory().unwrap();

        let mode: String = db
            .with_connection(|conn| {
                Ok(conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?)
            })
            .unwrap();
        // In-memory databases use memory journal, file databases use WAL
        assert!(mode == "memory" || mode == "wal");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let db = FinanceDatabase::in_memory().unwrap();

        let enabled: i64 = db
            .with_connection(|conn| {
                Ok(conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(enabled, 1);
    }

    #[test]
    fn test_clone_shares_connection() {
        let db1 = FinanceDatabase::in_memory().unwrap();
        let db2 = db1.clone();

        // Both should share the same connection (Arc)
        assert!(Arc::ptr_eq(&db1.conn, &db2.conn));
    }

    #[test]
    fn test_with_transaction() {
        let db = FinanceDatabase::in_memory().unwrap();

        // Insert an account using transaction
        let id = db
            .with_transaction(|tx| {
                tx.execute(
                    "INSERT INTO accounts (name, account_type, currency, balance, initial_balance, status)
                     VALUES ('Test', 'bank', 'CNY', 1000, 1000, 'active')",
                    [],
                )?;
                Ok(tx.last_insert_rowid())
            })
            .unwrap();

        assert!(id > 0);

        // Verify the account exists
        let name: String = db
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT name FROM accounts WHERE id = ?",
                    [id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(name, "Test");
    }
}
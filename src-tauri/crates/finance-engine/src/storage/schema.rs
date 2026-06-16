//! Database schema definition for SQLite storage
//!
//! Note: SQLite doesn't have native DECIMAL type. We use TEXT for decimal fields
//! to preserve precision and allow proper Decimal parsing.

/// Database schema for finance engine
pub struct DatabaseSchema;

impl DatabaseSchema {
    /// Get SQL for creating all tables
    pub fn create_tables_sql() -> Vec<String> {
        vec![
            Self::create_accounts_table(),
            Self::create_transactions_table(),
            Self::create_journal_entries_table(),
            Self::create_categories_table(),
            Self::create_budgets_table(),
            Self::create_budget_items_table(),
            Self::create_currencies_table(),
        ]
    }

    /// Accounts table
    fn create_accounts_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CNY',
    balance TEXT NOT NULL DEFAULT '0',
    initial_balance TEXT NOT NULL DEFAULT '0',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#.to_string()
    }

    /// Transactions table
    fn create_transactions_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'CNY',
    from_account_id INTEGER,
    to_account_id INTEGER,
    category_id INTEGER,
    reference TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_account_id) REFERENCES accounts(id),
    FOREIGN KEY (to_account_id) REFERENCES accounts(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
"#.to_string()
    }

    /// Journal entries table
    fn create_journal_entries_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS journal_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    debit TEXT NOT NULL DEFAULT '0',
    credit TEXT NOT NULL DEFAULT '0',
    description TEXT NOT NULL,
    date TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (transaction_id) REFERENCES transactions(id),
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
"#.to_string()
    }

    /// Categories table
    fn create_categories_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    category_type TEXT NOT NULL,
    parent_id INTEGER,
    icon TEXT,
    color TEXT,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);
"#.to_string()
    }

    /// Budgets table
    fn create_budgets_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    period TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    total_allocated TEXT NOT NULL DEFAULT '0',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#.to_string()
    }

    /// Budget items table
    fn create_budget_items_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS budget_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    allocated TEXT NOT NULL,
    spent TEXT NOT NULL DEFAULT '0',
    remaining TEXT NOT NULL DEFAULT '0',
    is_exceeded INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (budget_id) REFERENCES budgets(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);
"#.to_string()
    }

    /// Currencies table
    fn create_currencies_table() -> String {
        r#"
CREATE TABLE IF NOT EXISTS currencies (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    exchange_rate TEXT NOT NULL DEFAULT '1',
    decimal_places INTEGER NOT NULL DEFAULT 2
);
"#.to_string()
    }

    /// Seed data SQL
    pub fn seed_data_sql() -> Vec<String> {
        vec![
            Self::seed_categories(),
            Self::seed_currencies(),
        ]
    }

    fn seed_categories() -> String {
        r#"
INSERT INTO categories (name, category_type, icon, color) VALUES
    ('工资', 'income', '💼', '#4CAF50'),
    ('奖金', 'income', '🎁', '#8BC34A'),
    ('投资收益', 'income', '📈', '#CDDC39'),
    ('餐饮', 'expense', '🍽️', '#FF5722'),
    ('交通', 'expense', '🚗', '#FF9800'),
    ('购物', 'expense', '🛒', '#E91E63'),
    ('娱乐', 'expense', '🎮', '#9C27B0'),
    ('医疗', 'expense', '🏥', '#00BCD4');
"#.to_string()
    }

    fn seed_currencies() -> String {
        r#"
INSERT INTO currencies (code, name, symbol, exchange_rate, decimal_places) VALUES
    ('CNY', 'Chinese Yuan', '¥', '1', 2),
    ('USD', 'US Dollar', '$', '7.2', 2),
    ('EUR', 'Euro', '€', '7.8', 2),
    ('GBP', 'British Pound', '£', '9.1', 2),
    ('JPY', 'Japanese Yen', '¥', '0.048', 0),
    ('HKD', 'Hong Kong Dollar', 'HK$', '0.92', 2);
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let sql = DatabaseSchema::create_tables_sql();
        assert_eq!(sql.len(), 7);

        // Check each table exists
        for table_sql in sql {
            assert!(table_sql.contains("CREATE TABLE"));
        }
    }
}
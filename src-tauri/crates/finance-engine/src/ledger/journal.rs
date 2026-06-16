//! Journal entry model for double-entry bookkeeping

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Journal entry (会计分录)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub transaction_id: i64,
    pub account_id: i64,
    pub debit: Decimal,
    pub credit: Decimal,
    pub description: String,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl JournalEntry {
    pub fn debit(id: i64, transaction_id: i64, account_id: i64, amount: Decimal, description: String, date: NaiveDate) -> Self {
        Self {
            id,
            transaction_id,
            account_id,
            debit: amount,
            credit: Decimal::ZERO,
            description,
            date,
            created_at: Utc::now(),
        }
    }

    pub fn credit(id: i64, transaction_id: i64, account_id: i64, amount: Decimal, description: String, date: NaiveDate) -> Self {
        Self {
            id,
            transaction_id,
            account_id,
            debit: Decimal::ZERO,
            credit: amount,
            description,
            date,
            created_at: Utc::now(),
        }
    }

    pub fn is_debit(&self) -> bool {
        self.debit > Decimal::ZERO && self.credit.is_zero()
    }

    pub fn is_credit(&self) -> bool {
        self.credit > Decimal::ZERO && self.debit.is_zero()
    }

    pub fn amount(&self) -> Decimal {
        if self.is_debit() { self.debit } else { self.credit }
    }
}
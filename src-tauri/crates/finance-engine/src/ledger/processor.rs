//! Ledger processor

use std::collections::HashMap;

use crate::error::FinanceResult;
use crate::models::Transaction;
use crate::types::TransactionType;
use super::journal::JournalEntry;
use super::validator::LedgerValidator;

pub struct LedgerProcessor;

impl LedgerProcessor {
    pub fn create_entries(
        transaction: &Transaction,
        next_id_start: i64,
    ) -> FinanceResult<Vec<JournalEntry>> {
        let entries = match transaction.transaction_type {
            TransactionType::Income => Self::create_income_entries(transaction, next_id_start),
            TransactionType::Expense => Self::create_expense_entries(transaction, next_id_start),
            TransactionType::Transfer => Self::create_transfer_entries(transaction, next_id_start),
            TransactionType::Adjustment => vec![],
        };

        if entries.len() >= 2 {
            LedgerValidator::validate_balance(&entries)?;
        }

        Ok(entries)
    }

    fn create_income_entries(transaction: &Transaction, id_start: i64) -> Vec<JournalEntry> {
        let to_account = transaction.to_account_id.unwrap_or(0);
        vec![
            JournalEntry::debit(id_start, transaction.id, to_account, transaction.amount, transaction.description.clone(), transaction.date),
            JournalEntry::credit(id_start + 1, transaction.id, 999001, transaction.amount, transaction.description.clone(), transaction.date),
        ]
    }

    fn create_expense_entries(transaction: &Transaction, id_start: i64) -> Vec<JournalEntry> {
        let from_account = transaction.from_account_id.unwrap_or(0);
        vec![
            JournalEntry::debit(id_start, transaction.id, 999002, transaction.amount, transaction.description.clone(), transaction.date),
            JournalEntry::credit(id_start + 1, transaction.id, from_account, transaction.amount, transaction.description.clone(), transaction.date),
        ]
    }

    fn create_transfer_entries(transaction: &Transaction, id_start: i64) -> Vec<JournalEntry> {
        let from_account = transaction.from_account_id.unwrap_or(0);
        let to_account = transaction.to_account_id.unwrap_or(0);
        vec![
            JournalEntry::debit(id_start, transaction.id, to_account, transaction.amount, transaction.description.clone(), transaction.date),
            JournalEntry::credit(id_start + 1, transaction.id, from_account, transaction.amount, transaction.description.clone(), transaction.date),
        ]
    }
}

pub struct LedgerEngine {
    entries: HashMap<i64, Vec<JournalEntry>>,
    next_entry_id: i64,
}

impl LedgerEngine {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_entry_id: 1,
        }
    }

    pub fn process_transaction(&mut self, transaction: &Transaction) -> FinanceResult<Vec<JournalEntry>> {
        let entries = LedgerProcessor::create_entries(transaction, self.next_entry_id)?;
        self.next_entry_id += entries.len() as i64;
        self.entries.insert(transaction.id, entries.clone());
        Ok(entries)
    }

    pub fn all_entries(&self) -> Vec<JournalEntry> {
        self.entries.values().flatten().cloned().collect()
    }
}

impl Default for LedgerEngine {
    fn default() -> Self {
        Self::new()
    }
}
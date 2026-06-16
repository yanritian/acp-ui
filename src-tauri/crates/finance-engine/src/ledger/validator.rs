//! Ledger validator - Double-entry balance validation

use rust_decimal::Decimal;
use crate::error::{FinanceError, FinanceResult};
use super::journal::JournalEntry;

pub struct LedgerValidator;

impl LedgerValidator {
    pub fn validate_balance(entries: &[JournalEntry]) -> FinanceResult<bool> {
        let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();

        if total_debit != total_credit {
            return Err(FinanceError::DoubleEntryImbalance {
                debit: total_debit.to_string(),
                credit: total_credit.to_string(),
            });
        }

        Ok(true)
    }

    pub fn is_balanced(entries: &[JournalEntry]) -> bool {
        let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();
        total_debit == total_credit
    }

    pub fn imbalance(entries: &[JournalEntry]) -> Decimal {
        let total_debit: Decimal = entries.iter().map(|e| e.debit).sum();
        let total_credit: Decimal = entries.iter().map(|e| e.credit).sum();
        total_debit - total_credit
    }
}
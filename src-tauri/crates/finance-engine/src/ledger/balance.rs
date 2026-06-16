//! Balance calculator

use rust_decimal::Decimal;

use super::journal::JournalEntry;

pub struct BalanceCalculator;

impl BalanceCalculator {
    pub fn totals(entries: &[JournalEntry]) -> (Decimal, Decimal) {
        let total_debit = entries.iter().map(|e| e.debit).sum();
        let total_credit = entries.iter().map(|e| e.credit).sum();
        (total_debit, total_credit)
    }
}
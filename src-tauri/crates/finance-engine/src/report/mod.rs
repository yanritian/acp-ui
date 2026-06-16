//! Report module - Financial statements generation

mod balance_sheet;
mod income_statement;
mod cash_flow;
mod generator;

pub use balance_sheet::{BalanceSheet, AccountBalance};
pub use income_statement::{IncomeStatement, CategoryTotal};
pub use cash_flow::CashFlowStatement;
pub use generator::{ReportGenerator, ReportServiceImpl};

// Re-export traits from crate level
pub use crate::traits::ReportService;
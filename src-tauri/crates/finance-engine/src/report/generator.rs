//! Report generator

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::error::FinanceResult;
use crate::traits::ReportService;
use crate::types::{DateRange, TransactionType, AccountType};
use crate::models::{Account, Transaction};
use super::balance_sheet::{BalanceSheet, AccountBalance};
use super::income_statement::{IncomeStatement, CategoryTotal};
use super::cash_flow::CashFlowStatement;

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn generate_balance_sheet(accounts: &[Account], as_of_date: NaiveDate) -> BalanceSheet {
        let mut sheet = BalanceSheet::new(as_of_date, "CNY".to_string());

        for account in accounts {
            let balance_item = AccountBalance {
                account_id: account.id,
                name: account.name.clone(),
                account_type: account.account_type.to_string(),
                balance: account.effective_balance(),
                currency: account.currency.clone(),
            };

            match account.account_type {
                AccountType::Asset | AccountType::Bank | AccountType::Cash => {
                    sheet = sheet.add_asset(balance_item);
                }
                AccountType::Liability | AccountType::Credit => {
                    sheet = sheet.add_liability(balance_item);
                }
                AccountType::Equity => {
                    sheet = sheet.add_equity(balance_item);
                }
                AccountType::Income | AccountType::Expense => {}
            }
        }

        sheet
    }

    pub fn generate_income_statement(
        transactions: &[Transaction],
        start_date: NaiveDate,
        end_date: NaiveDate,
        categories: &HashMap<i64, String>,
    ) -> IncomeStatement {
        let mut statement = IncomeStatement::new(start_date, end_date, "CNY".to_string());
        let mut income_totals: HashMap<i64, Decimal> = HashMap::new();
        let mut expense_totals: HashMap<i64, Decimal> = HashMap::new();

        for tx in transactions {
            if tx.date >= start_date && tx.date <= end_date && tx.affects_balance() {
                match tx.transaction_type {
                    TransactionType::Income => {
                        if let Some(cat_id) = tx.category_id {
                            let current = income_totals.get(&cat_id).copied().unwrap_or(Decimal::ZERO);
                            income_totals.insert(cat_id, current + tx.amount);
                        }
                    }
                    TransactionType::Expense => {
                        if let Some(cat_id) = tx.category_id {
                            let current = expense_totals.get(&cat_id).copied().unwrap_or(Decimal::ZERO);
                            expense_totals.insert(cat_id, current + tx.amount);
                        }
                    }
                    _ => {}
                }
            }
        }

        for (cat_id, amount) in &income_totals {
            let name = categories.get(cat_id).cloned().unwrap_or_else(|| "其他收入".to_string());
            statement = statement.add_income(CategoryTotal {
                category_id: *cat_id,
                name,
                amount: *amount,
                percentage: 0.0,
                currency: "CNY".to_string(),
            });
        }

        for (cat_id, amount) in &expense_totals {
            let name = categories.get(cat_id).cloned().unwrap_or_else(|| "其他支出".to_string());
            statement = statement.add_expense(CategoryTotal {
                category_id: *cat_id,
                name,
                amount: *amount,
                percentage: 0.0,
                currency: "CNY".to_string(),
            });
        }

        statement
    }
}

pub struct ReportServiceImpl;

#[async_trait]
impl ReportService for ReportServiceImpl {
    async fn balance_sheet(&self, as_of_date: NaiveDate) -> FinanceResult<BalanceSheet> {
        Ok(BalanceSheet::new(as_of_date, "CNY".to_string()))
    }

    async fn income_statement(&self, range: DateRange) -> FinanceResult<IncomeStatement> {
        Ok(IncomeStatement::new(range.start, range.end, "CNY".to_string()))
    }

    async fn cash_flow_statement(&self, range: DateRange) -> FinanceResult<CashFlowStatement> {
        Ok(CashFlowStatement::new(range.start, range.end, "CNY".to_string(), Decimal::ZERO))
    }
}
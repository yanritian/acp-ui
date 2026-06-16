//! Income statement model (损益表/利润表)
//! Net Profit = Total Income - Total Expenses

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Category total for income statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTotal {
    /// Category ID
    pub category_id: i64,

    /// Category name
    pub name: String,

    /// Total amount
    pub amount: Decimal,

    /// Percentage of total
    pub percentage: f64,

    /// Currency
    pub currency: String,
}

/// Income statement (损益表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatement {
    /// Report start date
    pub start_date: NaiveDate,

    /// Report end date
    pub end_date: NaiveDate,

    /// Income categories and totals
    pub income: Vec<CategoryTotal>,

    /// Expense categories and totals
    pub expenses: Vec<CategoryTotal>,

    /// Total income
    pub total_income: Decimal,

    /// Total expenses
    pub total_expenses: Decimal,

    /// Net profit (income - expenses)
    pub net_profit: Decimal,

    /// Currency
    pub currency: String,

    /// Profit margin percentage
    pub profit_margin: f64,
}

impl IncomeStatement {
    /// Create a new income statement
    pub fn new(start_date: NaiveDate, end_date: NaiveDate, currency: String) -> Self {
        Self {
            start_date,
            end_date,
            income: Vec::new(),
            expenses: Vec::new(),
            total_income: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            net_profit: Decimal::ZERO,
            currency,
            profit_margin: 0.0,
        }
    }

    /// Add income category
    pub fn add_income(mut self, category: CategoryTotal) -> Self {
        self.income.push(category.clone());
        self.total_income += category.amount;
        self.calculate_profit();
        self
    }

    /// Add expense category
    pub fn add_expense(mut self, category: CategoryTotal) -> Self {
        self.expenses.push(category.clone());
        self.total_expenses += category.amount;
        self.calculate_profit();
        self
    }

    /// Calculate net profit and margins
    fn calculate_profit(&mut self) {
        self.net_profit = self.total_income - self.total_expenses;

        // Calculate profit margin
        if !self.total_income.is_zero() {
            self.profit_margin = (self.net_profit / self.total_income)
                .try_into()
                .unwrap_or(0.0)
                * 100.0;
        } else {
            self.profit_margin = 0.0;
        }

        // Recalculate percentages for categories
        self.recalculate_percentages();
    }

    /// Recalculate category percentages
    fn recalculate_percentages(&mut self) {
        for cat in &mut self.income {
            if !self.total_income.is_zero() {
                cat.percentage = (cat.amount / self.total_income)
                    .try_into()
                    .unwrap_or(0.0)
                    * 100.0;
            }
        }

        for cat in &mut self.expenses {
            if !self.total_expenses.is_zero() {
                cat.percentage = (cat.amount / self.total_expenses)
                    .try_into()
                    .unwrap_or(0.0)
                    * 100.0;
            }
        }
    }

    /// Check if profitable
    pub fn is_profitable(&self) -> bool {
        self.net_profit > Decimal::ZERO
    }

    /// Get expense ratio (expenses / income)
    pub fn expense_ratio(&self) -> f64 {
        if !self.total_income.is_zero() {
            (self.total_expenses / self.total_income)
                .try_into()
                .unwrap_or(0.0)
                * 100.0
        } else {
            0.0
        }
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "损益表 ({} - {})\n\
            收入总计: {} {}\n\
            支出总计: {} {}\n\
            净利润: {} {}\n\
            利润率: {:.1}%\n\
            状态: {}",
            self.start_date, self.end_date,
            self.total_income, self.currency,
            self.total_expenses, self.currency,
            self.net_profit, self.currency,
            self.profit_margin,
            if self.is_profitable() { "盈利 ✓" } else { "亏损 ✗" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn test_range() -> (NaiveDate, NaiveDate) {
        (
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
    }

    #[test]
    fn test_profitable_statement() {
        let (start, end) = test_range();
        let statement = IncomeStatement::new(start, end, "CNY".to_string())
            .add_income(CategoryTotal {
                category_id: 1,
                name: "工资".to_string(),
                amount: dec!(50000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            })
            .add_expense(CategoryTotal {
                category_id: 2,
                name: "餐饮".to_string(),
                amount: dec!(10000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            });

        assert!(statement.is_profitable());
        assert_eq!(statement.total_income, dec!(50000));
        assert_eq!(statement.total_expenses, dec!(10000));
        assert_eq!(statement.net_profit, dec!(40000));
        assert_eq!(statement.profit_margin, 80.0);
    }

    #[test]
    fn test_loss_statement() {
        let (start, end) = test_range();
        let statement = IncomeStatement::new(start, end, "CNY".to_string())
            .add_income(CategoryTotal {
                category_id: 1,
                name: "收入".to_string(),
                amount: dec!(5000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            })
            .add_expense(CategoryTotal {
                category_id: 2,
                name: "支出".to_string(),
                amount: dec!(8000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            });

        assert!(!statement.is_profitable());
        assert_eq!(statement.net_profit, dec!(-3000));
    }

    #[test]
    fn test_category_percentages() {
        let (start, end) = test_range();
        let statement = IncomeStatement::new(start, end, "CNY".to_string())
            .add_income(CategoryTotal {
                category_id: 1,
                name: "工资".to_string(),
                amount: dec!(30000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            })
            .add_income(CategoryTotal {
                category_id: 2,
                name: "奖金".to_string(),
                amount: dec!(20000),
                percentage: 0.0,
                currency: "CNY".to_string(),
            });

        // Total income = 50000, 工资 = 60%, 奖金 = 40%
        assert_eq!(statement.income[0].percentage, 60.0);
        assert_eq!(statement.income[1].percentage, 40.0);
    }
}
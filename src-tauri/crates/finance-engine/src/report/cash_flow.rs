//! Cash flow statement model (现金流量表)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Cash flow activity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CashFlowActivity {
    Operating,
    Investing,
    Financing,
}

/// Cash flow item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowItem {
    /// Item description
    pub description: String,

    /// Activity type
    pub activity: CashFlowActivity,

    /// Amount (positive = inflow, negative = outflow)
    pub amount: Decimal,

    /// Currency
    pub currency: String,
}

/// Cash flow statement (现金流量表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatement {
    /// Report start date
    pub start_date: NaiveDate,

    /// Report end date
    pub end_date: NaiveDate,

    /// Operating activities cash flow
    pub operating_activities: Vec<CashFlowItem>,

    /// Investing activities cash flow
    pub investing_activities: Vec<CashFlowItem>,

    /// Financing activities cash flow
    pub financing_activities: Vec<CashFlowItem>,

    /// Net cash from operating activities
    pub net_operating: Decimal,

    /// Net cash from investing activities
    pub net_investing: Decimal,

    /// Net cash from financing activities
    pub net_financing: Decimal,

    /// Total net cash flow
    pub net_cash_flow: Decimal,

    /// Beginning cash balance
    pub beginning_cash: Decimal,

    /// Ending cash balance
    pub ending_cash: Decimal,

    /// Currency
    pub currency: String,

    /// Whether the statement reconciles (Ending = Beginning + Net Cash Flow)
    pub is_reconciled: bool,
}

impl CashFlowStatement {
    /// Create a new cash flow statement
    pub fn new(start_date: NaiveDate, end_date: NaiveDate, currency: String, beginning_cash: Decimal) -> Self {
        Self {
            start_date,
            end_date,
            operating_activities: Vec::new(),
            investing_activities: Vec::new(),
            financing_activities: Vec::new(),
            net_operating: Decimal::ZERO,
            net_investing: Decimal::ZERO,
            net_financing: Decimal::ZERO,
            net_cash_flow: Decimal::ZERO,
            beginning_cash,
            ending_cash: beginning_cash,
            currency,
            is_reconciled: true,
        }
    }

    /// Add an operating activity item
    pub fn add_operating(mut self, item: CashFlowItem) -> Self {
        self.operating_activities.push(item.clone());
        self.net_operating += item.amount;
        self.calculate_total();
        self
    }

    /// Add an investing activity item
    pub fn add_investing(mut self, item: CashFlowItem) -> Self {
        self.investing_activities.push(item.clone());
        self.net_investing += item.amount;
        self.calculate_total();
        self
    }

    /// Add a financing activity item
    pub fn add_financing(mut self, item: CashFlowItem) -> Self {
        self.financing_activities.push(item.clone());
        self.net_financing += item.amount;
        self.calculate_total();
        self
    }

    /// Calculate totals and ending balance
    fn calculate_total(&mut self) {
        self.net_cash_flow = self.net_operating + self.net_investing + self.net_financing;
        self.ending_cash = self.beginning_cash + self.net_cash_flow;
        self.is_reconciled = true; // Always reconciled if calculated correctly
    }

    /// Check if cash position improved
    pub fn cash_improved(&self) -> bool {
        self.ending_cash > self.beginning_cash
    }

    /// Get cash change percentage
    pub fn cash_change_percentage(&self) -> f64 {
        if !self.beginning_cash.is_zero() {
            ((self.ending_cash - self.beginning_cash) / self.beginning_cash)
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
            "现金流量表 ({} - {})\n\
            经营活动净现金流: {} {}\n\
            投资活动净现金流: {} {}\n\
            筹资活动净现金流: {} {}\n\
            净现金流总计: {} {}\n\
            期初现金: {} {}\n\
            期末现金: {} {}\n\
            现金变化: {:.1}%\n\
            状态: {}",
            self.start_date, self.end_date,
            self.net_operating, self.currency,
            self.net_investing, self.currency,
            self.net_financing, self.currency,
            self.net_cash_flow, self.currency,
            self.beginning_cash, self.currency,
            self.ending_cash, self.currency,
            self.cash_change_percentage(),
            if self.cash_improved() { "增加 ✓" } else { "减少 ✗" }
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
    fn test_cash_flow_positive() {
        let (start, end) = test_range();
        let statement = CashFlowStatement::new(start, end, "CNY".to_string(), dec!(10000))
            .add_operating(CashFlowItem {
                description: "销售收入".to_string(),
                activity: CashFlowActivity::Operating,
                amount: dec!(50000),
                currency: "CNY".to_string(),
            })
            .add_operating(CashFlowItem {
                description: "工资支出".to_string(),
                activity: CashFlowActivity::Operating,
                amount: dec!(-20000),
                currency: "CNY".to_string(),
            })
            .add_investing(CashFlowItem {
                description: "购买设备".to_string(),
                activity: CashFlowActivity::Investing,
                amount: dec!(-5000),
                currency: "CNY".to_string(),
            });

        assert_eq!(statement.net_operating, dec!(30000));
        assert_eq!(statement.net_investing, dec!(-5000));
        assert_eq!(statement.net_cash_flow, dec!(25000));
        assert_eq!(statement.ending_cash, dec!(35000));
        assert!(statement.cash_improved());
    }

    #[test]
    fn test_cash_flow_negative() {
        let (start, end) = test_range();
        let statement = CashFlowStatement::new(start, end, "CNY".to_string(), dec!(10000))
            .add_operating(CashFlowItem {
                description: "支出".to_string(),
                activity: CashFlowActivity::Operating,
                amount: dec!(-15000),
                currency: "CNY".to_string(),
            });

        assert!(!statement.cash_improved());
        assert_eq!(statement.ending_cash, dec!(-5000));
    }
}
//! Financial Reports Module - 报表生成
//!
//! 资产负债表、利润表、现金流量表

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use crate::finance::accounting::{Ledger, ASSET_CASH, ASSET_BANK, LIABILITY_PAYABLE, EQUITY_CAPITAL, REVENUE_SALES, EXPENSE_COST};

/// 资产负债表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub company_name: String,
    pub report_date: DateTime<Utc>,
    pub period: String,
    /// 资产项目
    pub assets: AssetSection,
    /// 负债项目
    pub liabilities: LiabilitySection,
    /// 所有者权益
    pub equity: EquitySection,
}

/// 资产部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSection {
    /// 流动资产
    pub current_assets: Vec<AssetItem>,
    /// 非流动资产
    pub non_current_assets: Vec<AssetItem>,
    /// 资产总计
    pub total: Decimal,
}

/// 资产项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetItem {
    pub name: String,
    pub account_code: String,
    pub amount: Decimal,
}

/// 负债部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilitySection {
    /// 流动负债
    pub current_liabilities: Vec<LiabilityItem>,
    /// 非流动负债
    pub non_current_liabilities: Vec<LiabilityItem>,
    /// 负债总计
    pub total: Decimal,
}

/// 负债项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityItem {
    pub name: String,
    pub account_code: String,
    pub amount: Decimal,
}

/// 所有者权益部分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquitySection {
    pub items: Vec<EquityItem>,
    pub total: Decimal,
}

/// 权益项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityItem {
    pub name: String,
    pub account_code: String,
    pub amount: Decimal,
}

/// 利润表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub company_name: String,
    pub report_date: DateTime<Utc>,
    pub period: String,
    /// 营业收入
    pub revenue: Decimal,
    /// 营业成本
    pub cost: Decimal,
    /// 营业利润
    pub operating_profit: Decimal,
    /// 净利润
    pub net_profit: Decimal,
}

/// 现金流量表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatement {
    pub company_name: String,
    pub report_date: DateTime<Utc>,
    pub period: String,
    /// 经营活动现金流
    pub operating_cash_flow: Decimal,
    /// 投资活动现金流
    pub investing_cash_flow: Decimal,
    /// 筹资活动现金流
    pub financing_cash_flow: Decimal,
    /// 现金净增加
    pub net_cash_change: Decimal,
}

impl BalanceSheet {
    /// 从总账生成资产负债表
    pub fn generate(ledger: &Ledger, company_name: String, period: String) -> Self {
        // 收集资产
        let current_assets = vec![
            AssetItem {
                name: "现金".to_string(),
                account_code: ASSET_CASH.to_string(),
                amount: ledger.get_balance(ASSET_CASH)
                    .map(|b| b.closing_balance)
                    .unwrap_or(Decimal::ZERO),
            },
            AssetItem {
                name: "银行存款".to_string(),
                account_code: ASSET_BANK.to_string(),
                amount: ledger.get_balance(ASSET_BANK)
                    .map(|b| b.closing_balance)
                    .unwrap_or(Decimal::ZERO),
            },
        ];

        let total_assets = current_assets.iter()
            .map(|a| a.amount)
            .sum();

        // 收集负债
        let current_liabilities = vec![
            LiabilityItem {
                name: "应付账款".to_string(),
                account_code: LIABILITY_PAYABLE.to_string(),
                amount: ledger.get_balance(LIABILITY_PAYABLE)
                    .map(|b| b.closing_balance.abs())
                    .unwrap_or(Decimal::ZERO),
            },
        ];

        let total_liabilities = current_liabilities.iter()
            .map(|l| l.amount)
            .sum();

        // 收集权益
        let equity_items = vec![
            EquityItem {
                name: "实收资本".to_string(),
                account_code: EQUITY_CAPITAL.to_string(),
                amount: ledger.get_balance(EQUITY_CAPITAL)
                    .map(|b| b.closing_balance)
                    .unwrap_or(Decimal::ZERO),
            },
        ];

        let total_equity = equity_items.iter()
            .map(|e| e.amount)
            .sum();

        Self {
            company_name,
            report_date: Utc::now(),
            period,
            assets: AssetSection {
                current_assets,
                non_current_assets: vec![],
                total: total_assets,
            },
            liabilities: LiabilitySection {
                current_liabilities,
                non_current_liabilities: vec![],
                total: total_liabilities,
            },
            equity: EquitySection {
                items: equity_items,
                total: total_equity,
            },
        }
    }

    /// 验证资产负债平衡
    pub fn validate(&self) -> bool {
        // 资产 = 负债 + 所有者权益
        self.assets.total == self.liabilities.total + self.equity.total
    }

    /// 输出报表摘要
    pub fn summary(&self) -> String {
        format!(
            "资产负债表 - {}\n期间: {}\n资产总计: {}\n负债总计: {}\n权益总计: {}\n平衡验证: {}",
            self.company_name,
            self.period,
            self.assets.total,
            self.liabilities.total,
            self.equity.total,
            if self.validate() { "✅ 通过" } else { "❌ 不平衡" }
        )
    }
}

impl IncomeStatement {
    /// 从总账生成利润表
    pub fn generate(ledger: &Ledger, company_name: String, period: String) -> Self {
        let revenue = ledger.get_balance(REVENUE_SALES)
            .map(|b| b.credit_total - b.debit_total)
            .unwrap_or(Decimal::ZERO);

        let cost = ledger.get_balance(EXPENSE_COST)
            .map(|b| b.debit_total - b.credit_total)
            .unwrap_or(Decimal::ZERO);

        let operating_profit = revenue - cost;
        let net_profit = operating_profit;

        Self {
            company_name,
            report_date: Utc::now(),
            period,
            revenue,
            cost,
            operating_profit,
            net_profit,
        }
    }

    /// 输出报表摘要
    pub fn summary(&self) -> String {
        format!(
            "利润表 - {}\n期间: {}\n营业收入: {}\n营业成本: {}\n营业利润: {}\n净利润: {}",
            self.company_name,
            self.period,
            self.revenue,
            self.cost,
            self.operating_profit,
            self.net_profit
        )
    }
}

impl CashFlowStatement {
    /// 生成现金流量表
    pub fn generate(ledger: &Ledger, company_name: String, period: String) -> Self {
        let cash_balance = ledger.get_balance(ASSET_CASH)
            .map(|b| b.closing_balance - b.opening_balance)
            .unwrap_or(Decimal::ZERO);

        Self {
            company_name,
            report_date: Utc::now(),
            period,
            operating_cash_flow: cash_balance,
            investing_cash_flow: Decimal::ZERO,
            financing_cash_flow: Decimal::ZERO,
            net_cash_change: cash_balance,
        }
    }

    /// 输出报表摘要
    pub fn summary(&self) -> String {
        format!(
            "现金流量表 - {}\n期间: {}\n经营活动: {}\n投资活动: {}\n筹资活动: {}\n现金净增: {}",
            self.company_name,
            self.period,
            self.operating_cash_flow,
            self.investing_cash_flow,
            self.financing_cash_flow,
            self.net_cash_change
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finance::accounting::Ledger;
    use rust_decimal::Decimal;

    #[test]
    fn test_balance_sheet_generation() {
        let mut ledger = Ledger::new();

        // 登记一笔收款
        ledger.update_balance(ASSET_CASH, Decimal::from(10000), Decimal::ZERO);
        ledger.update_balance(REVENUE_SALES, Decimal::ZERO, Decimal::from(10000));

        // 生成报表
        let sheet = BalanceSheet::generate(&ledger, "测试公司".to_string(), "2024-01".to_string());

        // 验证
        assert_eq!(sheet.assets.total, Decimal::from(10000));
        println!("{}", sheet.summary());
    }

    #[test]
    fn test_income_statement_generation() {
        let mut ledger = Ledger::new();

        // 登记收入
        ledger.update_balance(REVENUE_SALES, Decimal::ZERO, Decimal::from(50000));
        ledger.update_balance(EXPENSE_COST, Decimal::from(30000), Decimal::ZERO);

        let income = IncomeStatement::generate(&ledger, "测试公司".to_string(), "2024-01".to_string());

        assert_eq!(income.revenue, Decimal::from(50000));
        assert_eq!(income.cost, Decimal::from(30000));
        assert_eq!(income.net_profit, Decimal::from(20000));
        println!("{}", income.summary());
    }

    #[test]
    fn test_all_reports() {
        let mut ledger = Ledger::new();

        // 模拟业务数据
        ledger.update_balance(ASSET_CASH, Decimal::from(100000), Decimal::ZERO);
        ledger.update_balance(REVENUE_SALES, Decimal::ZERO, Decimal::from(80000));
        ledger.update_balance(EXPENSE_COST, Decimal::from(50000), Decimal::ZERO);
        ledger.update_balance(EQUITY_CAPITAL, Decimal::ZERO, Decimal::from(50000));

        // 生成三大报表
        let balance = BalanceSheet::generate(&ledger, "ERP测试公司".to_string(), "2024-Q1".to_string());
        let income = IncomeStatement::generate(&ledger, "ERP测试公司".to_string(), "2024-Q1".to_string());
        let cashflow = CashFlowStatement::generate(&ledger, "ERP测试公司".to_string(), "2024-Q1".to_string());

        println!("\n=== ERP财务报表 ===\n");
        println!("{}\n", balance.summary());
        println!("{}\n", income.summary());
        println!("{}\n", cashflow.summary());
    }
}
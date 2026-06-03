//! Accounting Module - 会计核算
//!
//! 双向记账、凭证管理、账簿维护

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 预定义科目代码
pub const ASSET_CASH: &str = "1001";
pub const ASSET_BANK: &str = "1002";
pub const LIABILITY_PAYABLE: &str = "2001";
pub const EQUITY_CAPITAL: &str = "3001";
pub const REVENUE_SALES: &str = "4001";
pub const EXPENSE_COST: &str = "5001";

/// 会计凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    /// 凭证号
    voucher_no: String,
    /// 凭证日期
    date: DateTime<Utc>,
    /// 摘要
    description: String,
    /// 借方分录
    debit_entries: Vec<JournalEntry>,
    /// 贷方分录
    credit_entries: Vec<JournalEntry>,
    /// 制单人
    prepared_by: String,
    /// 审核人
    approved_by: Option<String>,
}

/// 日记账分录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// 科目代码
    pub account: String,
    /// 金额
    pub amount: Decimal,
    /// 摘要
    pub summary: String,
}

/// 总账
#[derive(Debug, Clone)]
pub struct Ledger {
    /// 科目余额
    balances: HashMap<String, AccountBalance>,
    /// 当前期间
    current_period: String,
}

/// 科目余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account: String,
    pub opening_balance: Decimal,
    pub debit_total: Decimal,
    pub credit_total: Decimal,
    pub closing_balance: Decimal,
}

/// 日记账
pub struct Journal {
    vouchers: Vec<Voucher>,
    is_closed: bool,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            vouchers: Vec::new(),
            is_closed: false,
        }
    }

    /// 登记凭证（双向记账）
    pub fn post_voucher(&mut self, voucher: Voucher) -> Result<(), String> {
        if self.is_closed {
            return Err("期间已结账，不能新增凭证".to_string());
        }

        // 验证借贷平衡
        let debit_total: Decimal = voucher.debit_entries.iter()
            .map(|e| e.amount)
            .sum();

        let credit_total: Decimal = voucher.credit_entries.iter()
            .map(|e| e.amount)
            .sum();

        if debit_total != credit_total {
            return Err(format!(
                "借贷不平衡: 借方{} != 贷方{}",
                debit_total, credit_total
            ));
        }

        self.vouchers.push(voucher);
        Ok(())
    }

    /// 获取凭证列表
    pub fn list_vouchers(&self) -> &[Voucher] {
        &self.vouchers
    }

    /// 期末结账
    pub fn close_period(&mut self) -> PeriodSummary {
        self.is_closed = true;

        let total_debit: Decimal = self.vouchers.iter()
            .flat_map(|v| v.debit_entries.iter())
            .map(|e| e.amount)
            .sum();

        let total_credit: Decimal = self.vouchers.iter()
            .flat_map(|v| v.credit_entries.iter())
            .map(|e| e.amount)
            .sum();

        PeriodSummary {
            voucher_count: self.vouchers.len(),
            total_debit,
            total_credit,
            closed_at: Utc::now(),
        }
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}

/// 期末汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodSummary {
    voucher_count: usize,
    total_debit: Decimal,
    total_credit: Decimal,
    closed_at: DateTime<Utc>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            current_period: "2024-01".to_string(),
        }
    }

    /// 更新科目余额
    pub fn update_balance(&mut self, account: &str, debit: Decimal, credit: Decimal) {
        let entry = self.balances.entry(account.to_string());

        entry.and_modify(|b| {
            b.debit_total += debit;
            b.credit_total += credit;
            b.closing_balance = b.opening_balance + b.debit_total - b.credit_total;
        }).or_insert(AccountBalance {
            account: account.to_string(),
            opening_balance: Decimal::ZERO,
            debit_total: debit,
            credit_total: credit,
            closing_balance: debit - credit,
        });
    }

    /// 获取科目余额
    pub fn get_balance(&self, account: &str) -> Option<&AccountBalance> {
        self.balances.get(account)
    }

    /// 试算平衡
    pub fn trial_balance(&self) -> TrialBalance {
        let accounts: Vec<AccountBalance> = self.balances.values().cloned().collect();

        let total_debit: Decimal = accounts.iter()
            .map(|a| a.debit_total)
            .sum();

        let total_credit: Decimal = accounts.iter()
            .map(|a| a.credit_total)
            .sum();

        TrialBalance {
            accounts,
            total_debit,
            total_credit,
            is_balanced: total_debit == total_credit,
        }
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

/// 试算平衡表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalance {
    accounts: Vec<AccountBalance>,
    total_debit: Decimal,
    total_credit: Decimal,
    is_balanced: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_entry_bookkeeping() {
        let mut journal = Journal::new();

        // 创建收款凭证：借现金，贷收入
        let voucher = Voucher {
            voucher_no: "V001".to_string(),
            date: Utc::now(),
            description: "销售收款".to_string(),
            debit_entries: vec![JournalEntry {
                account: ASSET_CASH.to_string(),
                amount: Decimal::from(10000),
                summary: "现金收入".to_string(),
            }],
            credit_entries: vec![JournalEntry {
                account: REVENUE_SALES.to_string(),
                amount: Decimal::from(10000),
                summary: "销售收入".to_string(),
            }],
            prepared_by: "系统".to_string(),
            approved_by: None,
        };

        // 登记凭证
        let result = journal.post_voucher(voucher);
        assert!(result.is_ok());
        assert_eq!(journal.vouchers.len(), 1);
    }

    #[test]
    fn test_balance_validation() {
        let mut journal = Journal::new();

        // 创建不平衡凭证
        let voucher = Voucher {
            voucher_no: "V002".to_string(),
            date: Utc::now(),
            description: "错误凭证".to_string(),
            debit_entries: vec![JournalEntry {
                account: ASSET_CASH.to_string(),
                amount: Decimal::from(100),
                summary: "借100".to_string(),
            }],
            credit_entries: vec![JournalEntry {
                account: LIABILITY_PAYABLE.to_string(),
                amount: Decimal::from(50), // 不平衡
                summary: "贷50".to_string(),
            }],
            prepared_by: "系统".to_string(),
            approved_by: None,
        };

        // 应拒绝
        let result = journal.post_voucher(voucher);
        assert!(result.is_err());
    }

    #[test]
    fn test_period_close() {
        let mut journal = Journal::new();

        // 登记凭证
        let voucher = Voucher {
            voucher_no: "V003".to_string(),
            date: Utc::now(),
            description: "测试".to_string(),
            debit_entries: vec![JournalEntry {
                account: ASSET_CASH.to_string(),
                amount: Decimal::from(1000),
                summary: "借".to_string(),
            }],
            credit_entries: vec![JournalEntry {
                account: REVENUE_SALES.to_string(),
                amount: Decimal::from(1000),
                summary: "贷".to_string(),
            }],
            prepared_by: "系统".to_string(),
            approved_by: None,
        };

        journal.post_voucher(voucher.clone()).unwrap();

        // 结账
        let summary = journal.close_period();
        assert_eq!(summary.voucher_count, 1);
        assert!(journal.is_closed);

        // 结账后不能新增
        let result = journal.post_voucher(voucher);
        assert!(result.is_err());
    }
}
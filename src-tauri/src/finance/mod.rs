//! ERP Finance Module
//!
//! 财务模块：会计核算 + 报表生成

pub mod accounting;
pub mod reports;

use serde::{Deserialize, Serialize};

/// 财务模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceConfig {
    pub company_name: String,
    pub currency: String,
    pub accounting_period: AccountingPeriod,
}

/// 会计期间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountingPeriod {
    Monthly,
    Quarterly,
    Yearly,
}

/// 财务错误
#[derive(Debug, thiserror::Error)]
pub enum FinanceError {
    #[error("借贷不平衡")]
    BalanceMismatch,
    #[error("凭证号重复")]
    DuplicateVoucher,
    #[error("账户不存在")]
    AccountNotFound,
    #[error("期间已结账")]
    PeriodClosed,
}
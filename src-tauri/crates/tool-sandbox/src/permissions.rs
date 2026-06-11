//! Permission checker - 权限检查器

use serde::{Deserialize, Serialize};

/// PermissionChecker - Tool执行权限检查
pub struct PermissionChecker {
    /// 允许的权限列表
    allowed_permissions: Vec<Permission>,
}

/// 权限类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    /// 文件读取
    FileRead,
    /// 文件写入
    FileWrite,
    /// 文件删除
    FileDelete,
    /// 命令执行
    CommandExecute,
    /// 网络请求
    NetworkRequest,
    /// 环境变量读取
    EnvRead,
    /// 环境变量写入
    EnvWrite,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            allowed_permissions: vec![
                Permission::FileRead,
                Permission::FileWrite,
                Permission::CommandExecute,
            ],
        }
    }

    pub fn with_permissions(permissions: Vec<Permission>) -> Self {
        Self {
            allowed_permissions: permissions,
        }
    }

    /// 检查权限是否被允许
    pub fn is_allowed(&self, permission: &Permission) -> bool {
        self.allowed_permissions.contains(permission)
    }

    /// 添加权限
    pub fn grant(&mut self, permission: Permission) {
        if !self.allowed_permissions.contains(&permission) {
            self.allowed_permissions.push(permission);
        }
    }

    /// 移除权限
    pub fn revoke(&mut self, permission: &Permission) {
        self.allowed_permissions.retain(|p| p != permission);
    }

    /// 获取所有权限
    pub fn list_permissions(&self) -> &[Permission] {
        &self.allowed_permissions
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}
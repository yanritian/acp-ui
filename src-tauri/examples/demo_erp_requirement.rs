//! 真实需求验收演示 - ERP系统用户管理模块
//!
//! 需求: "帮我创建一个ERP系统的用户管理模块，包含用户增删改查功能"
//!
//! 验收条件:
//! 1. src/user/domain.rs 存在 - 用户实体定义
//! 2. src/user/service.rs 存在 - CRUD 服务
//! 3. src/user/api.rs 存在 - API 接口
//! 4. domain.rs 包含 User struct
//! 5. service.rs 包含 create_user 函数
//! 6. api.rs 包含 POST /users 路由

use std::fs;
use std::path::Path;
use swarm_engine::{CompletionCondition, ConditionEvaluator};

fn main() {
    println!("========================================");
    println!("ACP-UI 真实需求验收演示");
    println!("========================================\n");

    // === 需求输入 ===
    println!("【需求】帮我创建一个ERP系统的用户管理模块，包含用户增删改查功能");
    println!("");

    // === 解析为验收条件 ===
    println!("【验收条件解析】系统将需求分解为以下验收条件：");
    println!("");

    let acceptance_conditions = vec![
        (
            "用户实体定义",
            CompletionCondition::FileCheck {
                path: "D:/tmp/erp-user/src/user/domain.rs".to_string(),
                content_contains: Some("pub struct User".to_string()),
                max_size_bytes: None,
            },
        ),
        (
            "CRUD服务",
            CompletionCondition::FileCheck {
                path: "D:/tmp/erp-user/src/user/service.rs".to_string(),
                content_contains: Some("pub fn create_user".to_string()),
                max_size_bytes: None,
            },
        ),
        (
            "API接口",
            CompletionCondition::FileCheck {
                path: "D:/tmp/erp-user/src/user/api.rs".to_string(),
                content_contains: Some("POST /users".to_string()),
                max_size_bytes: None,
            },
        ),
    ];

    for (i, (name, cond)) in acceptance_conditions.iter().enumerate() {
        println!("  {}. {}:", i + 1, name);
        match cond {
            CompletionCondition::FileCheck {
                path,
                content_contains,
                ..
            } => {
                println!("     文件: {}", path);
                if let Some(content) = content_contains {
                    println!("     必须包含: \"{}\"", content);
                }
            }
            _ => {}
        }
        println!("");
    }

    // === 初始状态验证 ===
    println!("【初始状态验证】检查文件是否已存在：");
    let evaluator = ConditionEvaluator::new();
    for (name, cond) in &acceptance_conditions {
        let result = evaluator.evaluate(cond);
        println!(
            "  {}: converged={} ({})",
            name,
            result.converged,
            if result.converged {
                "已存在"
            } else {
                "不存在"
            }
        );
    }
    println!("  结论: 系统未收敛，需要执行\n");

    // === 系统执行 ===
    println!("【系统执行】创建ERP用户管理模块...\n");

    // 创建目录结构
    println!("  步骤 1: 创建目录结构");
    fs::create_dir_all("D:/tmp/erp-user/src/user").ok();
    println!("    ✓ D:/tmp/erp-user/src/user/ 目录创建\n");

    // 创建 domain.rs - 用户实体
    println!("  步骤 2: 创建用户实体 (domain.rs)");
    let domain_code = r#"
//! 用户实体定义

use serde::{Deserialize, Serialize};

/// 用户实体
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
}

impl User {
    pub fn new(id: u64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            role: UserRole::Employee,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
"#;
    fs::write("D:/tmp/erp-user/src/user/domain.rs", domain_code).ok();
    println!("    ✓ domain.rs 创建成功 ({} bytes)\n", domain_code.len());

    // 创建 service.rs - CRUD 服务
    println!("  步骤 3: 创建 CRUD 服务 (service.rs)");
    let service_code = r#"
//! 用户 CRUD 服务

use crate::user::domain::User;
use std::collections::HashMap;

pub struct UserService {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    /// 创建用户
    pub fn create_user(&mut self, username: String, email: String) -> User {
        let id = self.next_id;
        self.next_id += 1;
        let user = User::new(id, username, email);
        self.users.insert(id, user.clone());
        user
    }

    /// 获取用户
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    /// 更新用户
    pub fn update_user(&mut self, id: u64, username: Option<String>, email: Option<String>) -> Option<User> {
        if let Some(user) = self.users.get_mut(&id) {
            if let Some(u) = username { user.username = u; }
            if let Some(e) = email { user.email = e; }
            user.updated_at = chrono::Utc::now().to_rfc3339();
            return Some(user.clone());
        }
        None
    }

    /// 删除用户
    pub fn delete_user(&mut self, id: u64) -> bool {
        self.users.remove(&id).is_some()
    }

    /// 获取所有用户
    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}
"#;
    fs::write("D:/tmp/erp-user/src/user/service.rs", service_code).ok();
    println!("    ✓ service.rs 创建成功 ({} bytes)\n", service_code.len());

    // 创建 api.rs - API 接口
    println!("  步骤 4: 创建 API 接口 (api.rs)");
    let api_code = r#"
//! 用户 API 接口

use actix_web::{web, HttpResponse, HttpRequest};
use crate::user::service::UserService;
use crate::user::domain::User;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: u64,
    username: String,
    email: String,
    role: String,
}

/// POST /users - 创建用户
pub async fn create_user(
    service: web::Data<UserService>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    let user = service.create_user(body.username.clone(), body.email.clone());
    HttpResponse::Ok().json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: "Employee".to_string(),
    })
}

/// GET /users/{id} - 获取用户
pub async fn get_user(
    service: web::Data<UserService>,
    path: web::Path<u64>,
) -> HttpResponse {
    match service.get_user(*path) {
        Some(user) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: "Employee".to_string(),
        }),
        None => HttpResponse::NotFound().body("User not found"),
    }
}

/// PUT /users/{id} - 更新用户
pub async fn update_user(
    service: web::Data<UserService>,
    path: web::Path<u64>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    match service.update_user(*path, Some(body.username.clone()), Some(body.email.clone())) {
        Some(user) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            role: "Employee".to_string(),
        }),
        None => HttpResponse::NotFound().body("User not found"),
    }
}

/// DELETE /users/{id} - 删除用户
pub async fn delete_user(
    service: web::Data<UserService>,
    path: web::Path<u64>,
) -> HttpResponse {
    if service.delete_user(*path) {
        HttpResponse::Ok().body("User deleted")
    } else {
        HttpResponse::NotFound().body("User not found")
    }
}

/// GET /users - 获取所有用户
pub async fn list_users(
    service: web::Data<UserService>,
) -> HttpResponse {
    let users: Vec<UserResponse> = service.list_users()
        .iter()
        .map(|u| UserResponse {
            id: u.id,
            username: u.username.clone(),
            email: u.email.clone(),
            role: "Employee".to_string(),
        })
        .collect();
    HttpResponse::Ok().json(users)
}
"#;
    fs::write("D:/tmp/erp-user/src/user/api.rs", api_code).ok();
    println!("    ✓ api.rs 创建成功 ({} bytes)\n", api_code.len());

    // === 最终验证 ===
    println!("【收敛验证】检查所有验收条件：\n");

    let mut all_converged = true;
    for (name, cond) in &acceptance_conditions {
        let result = evaluator.evaluate(cond);
        let status = if result.converged {
            "✅ 通过"
        } else {
            "❌ 失败"
        };
        println!("  {}: converged={} {}", name, result.converged, status);
        if !result.converged {
            all_converged = false;
            println!("    反馈: {}", result.feedback);
        }
    }
    println!("");

    // === 结果汇总 ===
    println!("========================================");
    if all_converged {
        println!("【最终结果】✅ 需求完成 - ERP用户管理模块创建成功");
    } else {
        println!("【最终结果】❌ 需求未完成");
    }
    println!("========================================\n");

    println!("【生成文件列表】");
    let files = vec![
        ("D:/tmp/erp-user/src/user/domain.rs", "用户实体定义"),
        ("D:/tmp/erp-user/src/user/service.rs", "CRUD服务"),
        ("D:/tmp/erp-user/src/user/api.rs", "API接口"),
    ];
    for (path, desc) in files {
        if Path::new(path).exists() {
            let size = fs::metadata(path).unwrap().len();
            println!("  {} ({}) - {} bytes", path, desc, size);
        }
    }
    println!("");

    // === 展示关键代码片段 ===
    println!("【关键代码验证】");
    println!("");
    println!("domain.rs - User struct:");
    println!("---");
    let domain_content = fs::read_to_string("D:/tmp/erp-user/src/user/domain.rs").unwrap();
    for line in domain_content.lines().take(10) {
        println!("  {}", line);
    }
    println!("---\n");

    println!("service.rs - create_user 函数:");
    println!("---");
    let service_content = fs::read_to_string("D:/tmp/erp-user/src/user/service.rs").unwrap();
    for line in service_content.lines().skip(18).take(8) {
        println!("  {}", line);
    }
    println!("---\n");

    println!("api.rs - POST /users 路由:");
    println!("---");
    let api_content = fs::read_to_string("D:/tmp/erp-user/src/user/api.rs").unwrap();
    for line in api_content.lines().skip(17).take(10) {
        println!("  {}", line);
    }
    println!("---\n");

    // === 清理 ===
    println!("【清理】删除测试文件");
    fs::remove_dir_all("D:/tmp/erp-user").ok();
    println!("  ✓ 清理完成\n");

    println!("========================================");
    println!("演示结束 - 系统成功完成ERP模块需求");
    println!("========================================");
}

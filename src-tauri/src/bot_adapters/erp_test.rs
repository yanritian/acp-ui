//! Bot Adapters ERP Integration Test
//!
//! 测试场景：通过Bot命令控制Agent开发ERP系统
//! 1. 发送 /status 查看系统状态
//! 2. 发送 /agent 创建ERP开发任务
//! 3. 发送 /team 多Agent协作开发
//! 4. 发送 /history 查看任务历史

use crate::bot_adapters::{format_response, parse_bot_text, BotCommand, BotResponse};

/// ERP系统开发测试场景
#[cfg(test)]
mod erp_test_scenarios {
    use super::*;

    /// 测试1: ERP系统状态查询
    #[test]
    fn test_erp_status_command() {
        // 模拟用户发送: /status
        let cmd = parse_bot_text("/status");
        assert!(matches!(cmd, BotCommand::Status));

        // 验证响应格式
        let response = BotResponse {
            success: true,
            message: "ERP系统开发环境状态".to_string(),
            data: Some(serde_json::json!({
                "project": "ERP System",
                "modules": ["inventory", "finance", "hr", "sales"],
                "status": "development",
                "agents_available": 3,
            })),
        };

        let telegram_output = format_response(&response, "telegram");
        assert!(telegram_output.contains("✅"));
        assert!(telegram_output.contains("ERP"));
        println!("Telegram输出: {}", telegram_output);
    }

    /// 测试2: 单Agent开发库存管理模块
    #[test]
    fn test_erp_inventory_development() {
        // 模拟用户发送: /agent 开发ERP库存管理模块，包含入库、出库、库存查询功能
        let cmd = parse_bot_text("/agent 开发ERP库存管理模块，包含入库、出库、库存查询功能");

        match cmd {
            BotCommand::Agent { prompt, agent_name } => {
                assert_eq!(prompt, "开发ERP库存管理模块，包含入库、出库、库存查询功能");
                assert!(agent_name.is_none());

                // 模拟Agent响应
                let response = BotResponse {
                    success: true,
                    message: "库存管理模块开发任务已创建".to_string(),
                    data: Some(serde_json::json!({
                        "module": "inventory",
                        "features": ["入库管理", "出库管理", "库存查询"],
                        "estimated_time": "2小时",
                        "files_to_create": [
                            "src/inventory/inbound.rs",
                            "src/inventory/outbound.rs",
                            "src/inventory/query.rs",
                        ],
                    })),
                };

                let feishu_output = format_response(&response, "feishu");
                assert!(feishu_output.contains("✅"));
                println!("Feishu输出: {}", feishu_output);
            }
            _ => panic!("期望Agent命令，收到: {:?}", cmd),
        }
    }

    /// 测试3: 多Agent协作开发ERP核心模块
    #[test]
    fn test_erp_multi_agent_collaboration() {
        // 模拟用户发送: /team codex,claude-code 开发ERP财务模块，需要会计核算和报表生成功能
        let cmd =
            parse_bot_text("/team codex,claude-code 开发ERP财务模块，需要会计核算和报表生成功能");

        match cmd {
            BotCommand::Team {
                prompt,
                agents,
                routing,
            } => {
                assert_eq!(prompt, "开发ERP财务模块，需要会计核算和报表生成功能");
                assert_eq!(agents, vec!["codex", "claude-code"]);
                assert_eq!(routing, "single");

                // 模拟多Agent协作响应
                let response = BotResponse {
                    success: true,
                    message: "ERP财务模块多Agent协作任务已启动".to_string(),
                    data: Some(serde_json::json!({
                        "module": "finance",
                        "agents": ["codex", "claude-code"],
                        "tasks": {
                            "codex": "负责会计核算核心逻辑",
                            "claude-code": "负责报表生成和UI",
                        },
                        "collaboration_mode": "parallel",
                    })),
                };

                let telegram_output = format_response(&response, "telegram");
                assert!(telegram_output.contains("codex"));
                assert!(telegram_output.contains("claude-code"));
                println!("Telegram输出: {}", telegram_output);
            }
            _ => panic!("期望Team命令，收到: {:?}", cmd),
        }
    }

    /// 测试4: 广播模式 - 所有Agent同时处理ERP需求
    #[test]
    fn test_erp_broadcast_mode() {
        // 模拟用户发送: /team broadcast 分析ERP系统架构设计
        let cmd = parse_bot_text("/team broadcast 分析ERP系统架构设计");

        match cmd {
            BotCommand::Team {
                prompt,
                agents,
                routing,
            } => {
                assert_eq!(prompt, "分析ERP系统架构设计");
                assert!(agents.is_empty());
                assert_eq!(routing, "broadcast");

                println!("ERP架构分析 - Broadcast模式触发成功");
            }
            _ => panic!("期望Team broadcast命令，收到: {:?}", cmd),
        }
    }

    /// 测试5: 任务暂停与恢复
    #[test]
    fn test_erp_task_control() {
        // 模拟暂停库存模块开发
        let pause_cmd = parse_bot_text("/pause inventory-agent-001");
        match pause_cmd {
            BotCommand::Pause { agent_id } => {
                assert_eq!(agent_id, "inventory-agent-001");
            }
            _ => panic!("期望Pause命令"),
        }

        // 模拟恢复
        let resume_cmd = parse_bot_text("/resume inventory-agent-001");
        match resume_cmd {
            BotCommand::Resume { agent_id } => {
                assert_eq!(agent_id, "inventory-agent-001");
            }
            _ => panic!("期望Resume命令"),
        }

        // 模拟取消财务模块任务
        let cancel_cmd = parse_bot_text("/cancel finance-task-002");
        match cancel_cmd {
            BotCommand::Cancel { target_id } => {
                assert_eq!(target_id, "finance-task-002");
            }
            _ => panic!("期望Cancel命令"),
        }
    }

    /// 测试6: 查看ERP开发历史
    #[test]
    fn test_erp_history_query() {
        // 模拟查询最近5条开发记录
        let cmd = parse_bot_text("/history 5");
        match cmd {
            BotCommand::History { limit } => {
                assert_eq!(limit, Some(5));
            }
            _ => panic!("期望History命令"),
        }

        // 默认查询10条
        let cmd = parse_bot_text("/history");
        match cmd {
            BotCommand::History { limit } => {
                assert_eq!(limit, Some(10));
            }
            _ => panic!("期望History命令"),
        }
    }

    /// 测试7: 完整ERP开发流程模拟
    #[test]
    fn test_erp_full_development_flow() {
        println!("\n=== ERP系统开发完整流程测试 ===\n");

        // Step 1: 检查状态
        let status_cmd = parse_bot_text("/status");
        println!("Step 1: 发送命令 /status -> {:?}", status_cmd);

        // Step 2: 查看可用Agent
        let agents_cmd = parse_bot_text("/agents");
        println!("Step 2: 发送命令 /agents -> {:?}", agents_cmd);

        // Step 3: 创建库存模块任务
        let inventory_cmd = parse_bot_text("/agent 开发库存管理模块");
        println!(
            "Step 3: 发送命令 /agent 开发库存管理模块 -> {:?}",
            inventory_cmd
        );

        // Step 4: 多Agent开发财务模块
        let finance_cmd = parse_bot_text("/team codex,claude-code 开发财务模块");
        println!(
            "Step 4: 发送命令 /team codex,claude-code 开发财务模块 -> {:?}",
            finance_cmd
        );

        // Step 5: 查看开发历史
        let history_cmd = parse_bot_text("/history 5");
        println!("Step 5: 发送命令 /history 5 -> {:?}", history_cmd);

        println!("\n=== ERP开发流程测试完成 ===\n");

        // 验证所有命令解析正确
        assert!(matches!(status_cmd, BotCommand::Status));
        assert!(matches!(agents_cmd, BotCommand::ListAgents));
        assert!(matches!(inventory_cmd, BotCommand::Agent { .. }));
        assert!(matches!(finance_cmd, BotCommand::Team { .. }));
        assert!(matches!(history_cmd, BotCommand::History { .. }));
    }

    /// 测试8: 各平台输出格式对比
    #[test]
    fn test_erp_cross_platform_output() {
        let erp_response = BotResponse {
            success: true,
            message: "ERP系统开发进度报告".to_string(),
            data: Some(serde_json::json!({
                "completed_modules": ["inventory", "hr"],
                "in_progress": ["finance"],
                "pending": ["sales", "purchase"],
                "total_progress": "40%",
            })),
        };

        println!("\n=== ERP进度报告 - 各平台输出格式 ===\n");

        // Telegram格式
        let telegram = format_response(&erp_response, "telegram");
        println!("Telegram:\n{}\n", telegram);

        // Feishu格式
        let feishu = format_response(&erp_response, "feishu");
        println!("Feishu:\n{}\n", feishu);

        // App WebSocket格式
        let app = format_response(&erp_response, "app");
        println!("App WebSocket:\n{}\n", app);

        // 验证各平台格式正确
        assert!(telegram.contains("✅"));
        assert!(telegram.contains("40%"));
        assert!(feishu.contains("✅"));
        assert_eq!(app, "ERP系统开发进度报告");
    }
}

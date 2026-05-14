# ACP-UI 多端协同Agent系统 - 完整架构设计

## 项目定位

**面向开发者的本地多Agent协作平台**

```
核心价值 = 
  Claude Code的沙箱能力 +
  Hermes的智能调度 +
  Trae的本地化版本 +
  多客户端开发工具支持
```

**差异化优势**：
- ✅ 完全本地（隐私保护，无云端依赖）
- ✅ 低延迟（局域网毫秒级响应）
- ✅ 完全控制（自定义MCP/Skills/Hooks）
- ✅ 多端协同（桌面+手机+CLI实时同步）
- ✅ 多客户端支持（HBuilderX/微信小程序/Android Studio/浏览器）

---

## 系统架构总览

### 极简版（MVP）

```
ACP-UI MVP
├── 多Agent进程管理 ✓ 已有
├── 基础WebSocket推送 ✓ 已有
├── ngrok隧道 ✓ 已有
├── 单一MCP配置（所有Agent共享）
├── 单一Skills配置（全局）
├── 无Hooks
├── 无Agent沙箱隔离
├── 无Hermes调度（手动切换Agent）
└── 手机端：基础日志显示
```

### 完整版（Ultimate）

```
ACP-UI Ultimate
├── 基础架构 ✓ 已有
│   ├── 多Agent进程管理
│   ├── ACP协议（JSON-RPC风格）
│   ├── WebSocket服务器
│   ├── ngrok隧道（真实进程管理）
│   └── SQLite数据库（TaskHistory/Error/Pattern等）
│
├── Agent沙箱系统 ⚠️ 需实现
│   ├── AgentConfig管理
│   ├── MCP隔离
│   ├── Skills隔离
│   ├── Hooks隔离
│   ├── 权限隔离
│   ├── 工作目录隔离
│   └── 环境变量隔离
│
├── Hermes智能调度 ⚠️ 需实现
│   ├── TaskParser（任务分解）
│   ├── AgentMatcher（Agent匹配）
│   ├── Orchestrator（协作编排）
│   └── Diagnostician（智能诊断）
│
├── 多端同步系统 ⚠️ 需增强
│   ├── LogStream系统（实时日志）
│   ├── ApiMonitor系统（网络请求监控）
│   ├── StatusSync系统（状态同步）
│   └── 手机端丰富UI组件库
│
├── 多客户端适配器 ⚠️ 需实现
│   ├── BrowserAdapter（Chrome DevTools Protocol）
│   ├── HBuilderXAdapter（CLI包装）
│   ├── WeChatDevToolsAdapter（DevTools + 代理）
│   ├── AndroidStudioAdapter（Gradle/ADB）
│   └── VSCodeAdapter（LSP/Extension）
│
└── 内置QA Agents ⚠️ 需实现
    ├── CodeReviewAgent（代码审查）
    ├── TestValidatorAgent（测试验证）
    ├── SecurityAuditorAgent（安全审计）
    └── ImplementationCheckerAgent（实现验证）
```

---

## Agent沙箱系统详细设计

### AgentConfig配置格式

```yaml
# agent-configs/hbuilder-agent.yaml
name: hbuilder-agent
description: "HBuilderX小程序开发Agent"

# MCP配置（Agent专属）
mcp_servers:
  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/miniprogram"]
    # 限制：只能访问指定目录

# Skills配置（Agent专属）
skills:
  - hbuilder-control       # 控制HBuilderX编译/运行
  - miniprogram-debug      # 小程序调试支持

# Hooks配置（Agent专属）
hooks:
  pre_tool_use:
    - script: check-hbuilder-status.sh
      # 检查HBuilderX是否运行
  post_tool_use:
    - script: notify-mobile.sh
      # 编译完成通知手机端
    - script: log-to-stream.sh
      # 输出到日志流

# 权限配置（Agent专属）
permissions:
  allow:
    - "Bash(hbuilder-cli)"         # 允许执行HBuilderX命令
    - "Bash(npm)"                  # 允许执行npm命令
    - "Read(/path/to/miniprogram/*)"  # 允许读取小程序目录
    - "Write(/path/to/miniprogram/*)" # 允许写入小程序目录
  deny:
    - "Bash(rm -rf)"               # 禁止危险命令
    - "Read(/etc/*)"               # 禁止读取系统目录

# 工作目录（强制限制）
cwd: "/path/to/miniprogram"

# 环境变量（Agent专属）
env:
  HBUILDER_PATH: "/Applications/HBuilderX.app"
  NODE_ENV: "development"
  MINIPROGRAM_ID: "wx1234567890"

# Agent能力标签（用于Hermes匹配）
capabilities:
  - compile          # 编译能力
  - miniprogram      # 小程序开发
  - hbuilder         # HBuilderX工具

# Agent元信息
metadata:
  author: "system"
  version: "1.0.0"
  created_at: "2026-05-10"
```

### MCP隔离实现方案

**方案A: 进程级MCP隔离**（推荐）

```
架构设计：
┌─────────────────────────────────────┐
│ 桌面端主进程                         │
│   ├── AgentManager                  │
│   │   ├── Agent A进程               │
│   │   │   ├── MCP连接1: filesystem  │
│   │   │   ├── MCP连接2: git        │
│   │   │   └── AcpSessionRunner      │
│   │   │                             │
│   │   ├── Agent B进程               │
│   │   │   ├── MCP连接3: devtools    │
│   │   │   └── AcpSessionRunner      │
│   │   │                             │
│   │   └── CodeReviewAgent进程       │
│   │       ├── MCP连接4: filesystem  │
│   │       └── AcpSessionRunner      │
│   │                                 │
│   └── HermesAgent（内置）            │
│       ├── 无MCP连接                 │
│       └── 只做调度逻辑              │
└─────────────────────────────────────┘

实现细节：
1. AgentConfigParser解析YAML配置
2. AgentLauncher启动Agent进程：
   - 根据mcp_servers配置启动MCP server进程
   - 建立MCP WebSocket连接
   - 注册MCP工具到AgentSession
3. MCP工具调用路由：
   - Agent调用MCP工具 → AcpSession → MCP连接 → MCP server
   - 结果返回 → AgentSession → Agent

关键代码：
```rust
// src-tauri/src/agent_launcher.rs
pub struct AgentLauncher {
    config_parser: AgentConfigParser,
    mcp_manager: McpProcessManager,
}

impl AgentLauncher {
    pub async fn launch_agent(&self, config_path: &str) -> Result<AgentProcess, String> {
        // 1. 解析配置
        let config = self.config_parser.parse(config_path)?;
        
        // 2. 启动MCP servers
        let mcp_connections = Vec::new();
        for (name, mcp_config) in &config.mcp_servers {
            let mcp_process = self.mcp_manager.start_mcp(name, mcp_config)?;
            let connection = McpConnection::connect(mcp_process.ws_url)?;
            mcp_connections.push(connection);
        }
        
        // 3. 启动Agent进程
        let agent_process = AgentProcess::spawn(&config)?;
        
        // 4. 注册MCP工具到AgentSession
        let session = AcpSessionRunner::new_with_mcp(agent_process, mcp_connections);
        
        Ok(session)
    }
}
```
```

### Skills隔离实现方案

```
Skills配置加载：
┌─────────────────────────────────────┐
│ Agent启动流程                        │
│   ├── 1. 解析AgentConfig            │
│   ├── 2. 加载指定Skills              │
│   │   ├── hbuilder-control.md       │
│   │   ├── miniprogram-debug.md      │
│   │   └── 只加载配置中指定的Skills   │
│   ├── 3. 注册Skills到AgentSession   │
│   └── 4. Agent只能调用注册的Skills  │
└─────────────────────────────────────┘

实现细节：
1. Skills存储：
   - 全局Skills目录：~/.claude/skills/（所有可用Skills）
   - Agent配置指定：skills: ["skill-a", "skill-b"]
   
2. Skills加载：
   - Agent启动时从全局目录加载指定Skills
   - 解析SKILL.md frontmatter
   - 注册到AgentSession.skills_registry
   
3. Skills调用：
   - Agent发送Skill调用命令
   - AcpSession检查skills_registry是否包含该Skill
   - 如果包含 → 执行Skill逻辑
   - 如果不包含 → 返回错误："Skill not available for this Agent"

关键代码：
```typescript
// src/lib/agent-runtime/skills-loader.ts
export class SkillsLoader {
    async loadSkillsForAgent(skillNames: string[]): Promise<SkillRegistry> {
        const registry = new SkillRegistry();
        
        for (const skillName of skillNames) {
            const skillPath = `~/.claude/skills/${skillName}/SKILL.md`;
            const skillContent = await fs.readFile(skillPath);
            const skill = parseSkillMarkdown(skillContent);
            registry.register(skillName, skill);
        }
        
        return registry;
    }
}

export class SkillRegistry {
    private skills: Map<string, Skill>;
    
    register(name: string, skill: Skill) {
        this.skills.set(name, skill);
    }
    
    has(name: string): boolean {
        return this.skills.has(name);
    }
    
    execute(name: string, context: SkillContext): Promise<SkillResult> {
        if (!this.has(name)) {
            throw new Error(`Skill '${name}' not available for this Agent`);
        }
        const skill = this.skills.get(name)!;
        return skill.execute(context);
    }
}
```
```

### Hooks隔离实现方案

```
Hooks执行架构：
┌─────────────────────────────────────┐
│ Agent操作流程                        │
│   ├── 用户请求                       │
│   ├── Agent选择工具                  │
│   ├── PreToolUse Hooks执行          │
│   │   ├── Agent A Hooks:            │
│   │   │   ├── check-status.sh       │
│   │   │   └── 结果：继续/阻断        │
│   │   └── 只执行Agent配置的Hooks     │
│   ├── 工具执行                       │
│   ├── PostToolUse Hooks执行         │
│   │   ├── Agent A Hooks:            │
│   │   │   ├── notify-mobile.sh      │
│   │   │   ├── log-to-stream.sh      │
│   │   └── 只执行Agent配置的Hooks     │
│   └── 返回结果                       │
└─────────────────────────────────────┘

实现细节：
1. Hooks存储：
   - 全局Hooks目录：~/.claude/hooks/（所有可用Hooks）
   - Agent配置指定：hooks: { pre: ["hook-a"], post: ["hook-b"] }
   
2. Hooks执行：
   - Agent进程内执行Hook脚本
   - 传递工具信息作为参数
   - 捕获Hook输出和退出码
   
3. Hooks结果处理：
   - PreToolUse退出码2 → 阻断工具执行
   - stderr有内容 → 显示警告但不阻断
   - stdout → 作为Hook输出记录

关键代码：
```rust
// src-tauri/src/hooks_executor.rs
pub struct HooksExecutor {
    hooks_dir: PathBuf,
}

impl HooksExecutor {
    pub async fn execute_pre_hooks(
        &self,
        agent_config: &AgentConfig,
        tool_name: &str,
        tool_params: &Value,
    ) -> Result<HooksResult, String> {
        let hooks = &agent_config.hooks.pre_tool_use;
        let mut result = HooksResult::Continue;
        
        for hook_script in hooks {
            let output = self.execute_hook_script(
                hook_script,
                tool_name,
                tool_params,
            )?;
            
            if output.exit_code == 2 {
                result = HooksResult::Block(output.stderr);
                break;
            } else if !output.stderr.is_empty() {
                // 显示警告，但不阻断
                result = HooksResult::Warn(output.stderr);
            }
        }
        
        Ok(result)
    }
    
    fn execute_hook_script(
        &self,
        script_name: &str,
        tool_name: &str,
        tool_params: &Value,
    ) -> Result<HookOutput, String> {
        let script_path = self.hooks_dir.join(script_name);
        let output = Command::new(&script_path)
            .arg(tool_name)
            .arg(tool_params.to_string())
            .output()
            .map_err(|e| e.to_string())?;
        
        Ok(HookOutput {
            exit_code: output.status.code().unwrap_or(0),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
```
```

### 权限隔离实现方案

```
权限检查流程：
┌─────────────────────────────────────┐
│ Agent执行命令前                      │
│   ├── 1. 捕获命令意图                │
│   │   ├── Bash: hbuilder-cli        │
│   │   ├── Read: /path/to/file       │
│   │   └── Write: /path/to/file      │
│   │                                 │
│   ├── 2. 权限匹配                    │
│   │   ├── allow规则匹配              │
│   │   │   ├── "Bash(hbuilder-cli)" ✓ │
│   │   │   ├── 正则匹配允许           │
│   │   │   └── 通过                   │
│   │   │                             │
│   │   ├── deny规则匹配               │
│   │   │   ├── "Bash(rm -rf)" ✗      │
│   │   │   ├── 匹配到禁止规则         │
│   │   │   └── 阻断                   │
│   │                                 │
│   ├── 3. 工作目录检查                │
│   │   ├── Read路径是否在cwd内？     │
│   │   ├── Write路径是否在cwd内？    │
│   │   └── 超出cwd → 阻断             │
│   │                                 │
│   ├── 4. 执行或阻断                  │
│   │   ├── 通过 → 执行命令            │
│   │   ├── 阻断 → 返回权限错误        │
│   └─────────────────────────────────┘
└─────────────────────────────────────┘

关键代码：
```rust
// src-tauri/src/permission_checker.rs
pub struct PermissionChecker {
    allow_patterns: Vec<Pattern>,
    deny_patterns: Vec<Pattern>,
    cwd: PathBuf,
}

impl PermissionChecker {
    pub fn check_permission(&self, tool_call: &ToolCall) -> Result<(), PermissionError> {
        let command_str = format!("{}({})", tool_call.tool, tool_call.params);
        
        // 1. 检查deny规则
        for deny_pattern in &self.deny_patterns {
            if deny_pattern.matches(&command_str) {
                return Err(PermissionError::Denied {
                    reason: format!("Command '{}' is denied by rule: {}", command_str, deny_pattern),
                });
            }
        }
        
        // 2. 检查allow规则
        let allowed = self.allow_patterns.iter().any(|p| p.matches(&command_str));
        if !allowed {
            return Err(PermissionError::NotAllowed {
                reason: format!("Command '{}' is not in allow list", command_str),
            });
        }
        
        // 3. 检查工作目录（针对文件操作）
        if tool_call.tool == "Read" || tool_call.tool == "Write" {
            let path = extract_path_from_params(&tool_call.params)?;
            if !self.is_within_cwd(&path) {
                return Err(PermissionError::OutsideCwd {
                    reason: format!("Path '{}' is outside working directory: {}", path, self.cwd),
                });
            }
        }
        
        Ok(())
    }
    
    fn is_within_cwd(&self, path: &Path) -> bool {
        let canonical_path = path.canonicalize().ok();
        let canonical_cwd = self.cwd.canonicalize().ok();
        
        match (canonical_path, canonical_cwd) {
            (Some(p), Some(c)) => p.starts_with(&c),
            _ => false,
        }
    }
}
```
```

---

## Hermes智能调度系统详细设计

### TaskParser任务分解器

```
任务分解流程：
┌─────────────────────────────────────┐
│ 用户输入："我要开发一个小程序功能"   │
│   ├── 1. AI意图分析                 │
│   │   ├── 输入：用户请求             │
│   │   ├── 输出：任务类型（开发任务） │
│   │   └── 算法：LLM分类 + 规则匹配   │
│   │                                 │
│   ├── 2. 任务模板匹配               │
│   │   ├── 任务类型：开发            │
│   │   ├── 匹配模板：miniprogram-dev │
│   │   └── 模板包含标准步骤          │
│   │                                 │
│   ├── 3. 任务分解                   │
│   │   ├── Task 1: 编写代码          │
│   │   │   ├── Agent: VS Code        │
│   │   │   ├── 依赖: 无              │
│   │   │   └── 优先级: 1             │
│   │   │                             │
│   │   ├── Task 2: 编译测试          │
│   │   │   ├── Agent: HBuilderX      │
│   │   │   ├── 依赖: Task 1          │
│   │   │   └── 优先级: 2             │
│   │   │                             │
│   │   ├── Task 3: 真机调试          │
│   │   │   ├── Agent: 微信调试       │
│   │   │   ├── 依赖: Task 2          │
│   │   │   └─ 优先级: 3             │
│   │   │                             │
│   │   ├── Task 4: API监控           │
│   │   │   ├── Agent: 微信调试       │
│   │   │   ├── 依赖: Task 3          │
│   │   │   └─ 优先级: 4             │
│   │   │                             │
│   │   ├── Task 5: Code Review       │
│   │   │   ├── Agent: CodeReviewAgent│
│   │   │   ├── 依赖: Task 1          │
│   │   │   └─ 优先级: 2             │
│   │   │                             │
│   │   └── Task 6: 测试验证          │
│   │   │   ├── Agent: TestValidatorAgent│
│   │   │   ├── 依赖: Task 2          │
│   │   │   └─ 优先级: 3             │
│   │                                 │
│   ├── 4. 生成任务图                 │
│   │   ├── DAG图（依赖关系）         │
│   │   ├── 并行任务识别              │
│   │   └── 执行顺序规划              │
│   │                                 │
│   └── 5. 返回任务列表               │
└─────────────────────────────────────┘

任务模板存储：
```yaml
# task-templates/miniprogram-dev.yaml
name: miniprogram-development
description: 小程序开发标准流程

tasks:
  - id: task-1
    name: 编写代码
    agent_requirements: [code-editor, filesystem]
    priority: 1
    dependencies: []
    
  - id: task-2
    name: 编译测试
    agent_requirements: [compile, hbuilder]
    priority: 2
    dependencies: [task-1]
    
  - id: task-3
    name: 真机调试
    agent_requirements: [debug, miniprogram]
    priority: 3
    dependencies: [task-2]
    
  - id: task-5
    name: Code Review
    agent_requirements: [code-review, security-audit]
    priority: 2  # 与task-2并行
    dependencies: [task-1]
    
  - id: task-6
    name: 测试验证
    agent_requirements: [test-validator]
    priority: 3  # 与task-3并行
    dependencies: [task-2]

# 任务类型识别规则
trigger_rules:
  keywords: ["小程序", "微信", "HBuilderX"]
  patterns: ["开发.*小程序", "创建.*微信"]
```

关键代码：
```typescript
// src/lib/hermes/task-parser.ts
export class TaskParser {
    private templates: Map<string, TaskTemplate>;
    
    async parseUserRequest(userRequest: string): Promise<TaskGraph> {
        // 1. AI意图分析
        const intent = await this.analyzeIntent(userRequest);
        
        // 2. 匹配任务模板
        const template = this.matchTemplate(intent);
        
        // 3. 实例化任务
        const tasks = template.tasks.map(t => ({
            ...t,
            status: 'pending',
            assigned_agent: null,
        }));
        
        // 4. 构建任务图
        const graph = this.buildDAG(tasks);
        
        return graph;
    }
    
    private async analyzeIntent(request: string): Promise<TaskIntent> {
        // 规则匹配 + LLM辅助
        for (const rule of this.triggerRules) {
            if (this.matchesRule(request, rule)) {
                return {
                    type: rule.task_type,
                    confidence: 0.9,
                };
            }
        }
        
        // LLM分类（备用）
        const llmResult = await this.llmClassify(request);
        return llmResult;
    }
    
    private buildDAG(tasks: Task[]): TaskGraph {
        const graph = new TaskGraph();
        
        // 添加节点
        for (const task of tasks) {
            graph.addNode(task.id, task);
        }
        
        // 添加边（依赖关系）
        for (const task of tasks) {
            for (const depId of task.dependencies) {
                graph.addEdge(depId, task.id);
            }
        }
        
        // 检测并行任务
        graph.detectParallelTasks();
        
        return graph;
    }
}
```
```

### AgentMatcher Agent匹配器

```
Agent能力向量匹配：
┌─────────────────────────────────────┐
│ Task需求分析                         │
│   ├── Task 2: 编译测试               │
│   ├── 需求能力：[compile, hbuilder]  │
│   └─────────────────────────────────┘
│                                     │
│ Agent能力库                          │
│   ├── Agent A (VS Code)              │
│   │   ├── 能力：[code-editor, filesystem] │
│   │   └── 匹配度：0.2（不匹配）      │
│   │                                 │
│   ├── Agent B (HBuilderX)            │
│   │   ├── 能力：[compile, hbuilder, miniprogram] │
│   │   └── 匹配度：0.85（高度匹配）   │
│   │                                 │
│   ├── Agent C (微信调试)             │
│   │   ├── 能力：[debug, miniprogram] │
│   │   └── 匹配度：0.4（部分匹配）    │
│   │                                 │
│   ├── CodeReviewAgent                │
│   │   ├── 能力：[code-review, security-audit] │
│   │   └─ 匹配度：0.0（不匹配）      │
│   │                                 │
│   ├── TestValidatorAgent             │
│   │   ├── 能力：[test-validator]     │
│   │   └─ 匹配度：0.0（不匹配）      │
│   │                                 │
│   └── 匹配结果：Agent B              │
└─────────────────────────────────────┘

匹配算法：
- Jaccard相似度：交集/并集
- 最小匹配阈值：0.6
- 优先选择匹配度最高的Agent
- 如果无匹配Agent → 动态创建或报错

关键代码：
```typescript
// src/lib/hermes/agent-matcher.ts
export class AgentMatcher {
    private agents: Map<string, AgentCapabilities>;
    
    matchAgent(taskRequirements: string[]): string | null {
        let bestMatch: { agentId: string, score: number } | null = null;
        
        for (const [agentId, capabilities] of this.agents) {
            const score = this.jaccardSimilarity(taskRequirements, capabilities.tags);
            
            if (score >= 0.6) {
                if (!bestMatch || score > bestMatch.score) {
                    bestMatch = { agentId, score };
                }
            }
        }
        
        return bestMatch?.agentId || null;
    }
    
    private jaccardSimilarity(setA: string[], setB: string[]): number {
        const intersection = setA.filter(x => setB.includes(x));
        const union = [...new Set([...setA, ...setB])];
        return intersection.length / union.length;
    }
}

export interface AgentCapabilities {
    id: string;
    name: string;
    tags: string[];  // 能力标签
    status: 'idle' | 'busy' | 'error';
    currentLoad: number;  // 当前任务数
}
```
```

### Orchestrator协作编排器

```
Agent协作流程：
┌─────────────────────────────────────┐
│ Hermes启动任务执行                   │
│   ├── 1. Task 1 → Agent A            │
│   │   ├── 分配任务                   │
│   │   ├── Agent A执行                │
│   │   └── TaskComplete事件           │
│   │                                 │
│   ├── 2. 监听TaskComplete            │
│   │   ├── Task 1完成                 │
│   │   ├── 检查Task 1的依赖者         │
│   │   ├── Task 2和Task 5可并行执行   │
│   │                                 │
│   ├── 3. Task 2 → Agent B            │
│   │   ├── 分配任务                   │
│   │   └── Agent B执行                │
│   │                                 │
│   ├── 4. Task 5 → CodeReviewAgent    │
│   │   ├── 分配任务                   │
│   │   ├── Agent执行代码审查          │
│   │   ├── ReviewResult事件           │
│   │   └─ 发现问题 → 阻断后续任务    │
│   │                                 │
│   ├── 5. 处理ReviewResult            │
│   │   ├── CodeReview发现问题         │
│   │   ├── 决策：阻断还是继续？       │
│   │   ├── 阻断 → 通知Agent A修复     │
│   │   └── 修复后重新执行             │
│   │                                 │
│   ├── 6. Task 2完成                  │
│   │   ├── 检查依赖者：Task 3, Task 6 │
│   │   ├── 并行执行                   │
│   │                                 │
│   ├── 7. Task 6 → TestValidatorAgent │
│   │   ├── 执行测试验证               │
│   │   ├── TestResult事件             │
│   │   └─ 发现测试失败 → 阻断        │
│   │                                 │
│   ├── 8. 最终状态汇总                │
│   │   ├── 所有任务完成               │
│   │   ├── 或：有任务失败             │
│   │   └── 通知用户                   │
└─────────────────────────────────────┘

关键代码：
```typescript
// src/lib/hermes/orchestrator.ts
export class Orchestrator {
    private eventBus: EventBus;
    private taskGraph: TaskGraph;
    private agentStatus: Map<string, AgentStatus>;
    
    async startExecution(graph: TaskGraph): Promise<void> {
        this.taskGraph = graph;
        
        // 注册事件监听
        this.eventBus.on('TaskComplete', this.handleTaskComplete.bind(this));
        this.eventBus.on('TaskFailed', this.handleTaskFailed.bind(this));
        this.eventBus.on('ReviewResult', this.handleReviewResult.bind(this));
        this.eventBus.on('TestResult', this.handleTestResult.bind(this));
        
        // 启动无依赖的任务
        const readyTasks = graph.getReadyTasks();
        for (const task of readyTasks) {
            this.assignTask(task);
        }
    }
    
    private async assignTask(task: Task): Promise<void> {
        const agentId = AgentMatcher.matchAgent(task.requirements);
        if (!agentId) {
            throw new Error(`No agent available for task: ${task.name}`);
        }
        
        // 发送TaskAssignment命令
        await this.sendCommand(agentId, {
            type: 'TaskAssignment',
            task_id: task.id,
            task_spec: task.spec,
        });
        
        this.agentStatus.set(agentId, { status: 'busy', currentTask: task.id });
    }
    
    private async handleTaskComplete(event: TaskCompleteEvent): Promise<void> {
        const taskId = event.task_id;
        
        // 更新任务状态
        this.taskGraph.markComplete(taskId);
        
        // 检查依赖此任务的后续任务
        const nextTasks = this.taskGraph.getDependents(taskId);
        const readyTasks = nextTasks.filter(t => this.taskGraph.isReady(t.id));
        
        // 并行分配可执行的任务
        for (const task of readyTasks) {
            await this.assignTask(task);
        }
        
        // 检查是否所有任务完成
        if (this.taskGraph.isAllComplete()) {
            this.emitEvent('AllTasksComplete', { result: 'success' });
        }
    }
    
    private async handleReviewResult(event: ReviewResultEvent): Promise<void> {
        // CodeReview发现问题
        if (event.has_issues) {
            // 决策：阻断后续任务
            const blockedTasks = this.taskGraph.getDependents(event.task_id);
            for (const task of blockedTasks) {
                this.taskGraph.markBlocked(task.id);
            }
            
            // 通知用户和原Agent修复
            this.emitEvent('CodeReviewBlocked', {
                issues: event.issues,
                original_task: event.related_task,
            });
        }
    }
    
    private async handleTestResult(event: TestResultEvent): Promise<void> {
        // 测试验证失败
        if (!event.passed) {
            // 阻断后续任务
            const blockedTasks = this.taskGraph.getDependents(event.task_id);
            for (const task of blockedTasks) {
                this.taskGraph.markBlocked(task.id);
            }
            
            // 通知用户
            this.emitEvent('TestValidationFailed', {
                failures: event.failures,
                original_task: event.related_task,
            });
        }
    }
}

export class EventBus {
    private listeners: Map<string, EventHandler[]>;
    
    on(eventType: string, handler: EventHandler) {
        if (!this.listeners.has(eventType)) {
            this.listeners.set(eventType, []);
        }
        this.listeners.get(eventType)!.push(handler);
    }
    
    emit(eventType: string, data: any) {
        const handlers = this.listeners.get(eventType) || [];
        for (const handler of handlers) {
            handler(data);
        }
    }
}
```
```

---

## 内置QA Agents详细设计

### CodeReviewAgent（代码审查Agent）

```yaml
# agent-configs/code-review-agent.yaml
name: code-review-agent
description: "代码审查Agent - 防止AI生成错误代码"

mcp_servers:
  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem"]

skills:
  - code-review        # 代码质量审查
  - security-review    # 安全漏洞检查

hooks:
  post_tool_use:
    - script: validate-code.sh      # 验证代码语法
    - script: check-best-practices.sh # 检查最佳实践

permissions:
  allow:
    - "Read(*)"         # 允许读取所有文件（审查需要）
    - "Bash(npx tsc)"   # 允许运行类型检查
    - "Bash(npx eslint)"# 允许运行lint检查
  deny:
    - "Write(*)"        # 禁止写入（审查Agent不修改代码）

cwd: "/tmp/code-review-session"

capabilities:
  - code-review
  - security-audit
  - syntax-validation
  - best-practices-check

# 审查规则配置
review_rules:
  # 必须检查的项目
  required_checks:
    - syntax_error        # 语法错误
    - type_error          # 类型错误
    - undefined_variable  # 未定义变量
    - unused_import       # 未使用导入
    - hardcoded_secret    # 硬编码密钥
    
  # 建议检查的项目
  suggested_checks:
    - code_style          # 代码风格
    - naming_convention   # 命名规范
    - duplication         # 代码重复
    - complexity          # 复杂度过高
    
  # 安全检查
  security_checks:
    - sql_injection       # SQL注入风险
    - xss_vulnerability   # XSS漏洞
    - unsafe_crypto       # 不安全加密
    - missing_auth        # 缺少认证
    
  # 审查结果阈值
  thresholds:
    critical: 0           # 关键错误必须为0
    high: 5               # 高优先级问题≤5
    medium: 10            # 中优先级问题≤10
    low: ignore           # 低优先级忽略
```

```
CodeReviewAgent执行流程：
┌─────────────────────────────────────┐
│ Hermes分配审查任务                   │
│   ├── TaskAssignment                │
│   │   ├── task_id: code-review-1    │
│   │   ├── target_files: [...]       │
│   │   └── review_type: full         │
│   │                                 │
│   ├── CodeReviewAgent接收           │
│   │   ├── 1. 读取目标文件            │
│   │   ├── 2. 运行类型检查            │
│   │   │   ├── npx tsc --noEmit      │
│   │   │   ├── 捕获错误输出           │
│   │   │   └── 解析错误信息           │
│   │   │                             │
│   │   ├── 3. 运行Lint检查            │
│   │   │   ├── npx eslint            │
│   │   │   ├── 捕获lint输出           │
│   │   │   └── 解析lint信息           │
│   │   │                             │
│   │   ├── 4. 安全扫描                │
│   │   │   ├── grep硬编码密钥         │
│   │   │   ├── grepSQL拼接            │
│   │   │   └── grepXSS风险            │
│   │   │                             │
│   │   ├── 5. 生成审查报告            │
│   │   │   ├── 分类问题               │
│   │   │   │   ├── Critical: 0       │
│   │   │   │   ├── High: 2           │
│   │   │   │   ├── Medium: 5         │
│   │   │   │   └ Low: 10            │
│   │   │   │                         │
│   │   │   ├── 每个问题：             │
│   │   │   │   ├── 文件位置           │
│   │   │   │   ├── 行号               │
│   │   │   │   ├── 问题描述           │
│   │   │   │   ├── 修复建议           │
│   │   │   │   └── 优先级             │
│   │   │   │                         │
│   │   │   ├── 整体评分               │
│   │   │   │   ├── Pass/Fail          │
│   │   │   │   ├── 评分：85/100       │
│   │   │   │   └─ 建议：修复High问题 │
│   │   │   │                         │
│   │   ├── 6. 发送ReviewResult事件   │
│   │   │   ├── has_issues: true      │
│   │   │   ├── issues: [...]         │
│   │   │   ├── overall_status: fail  │
│   │   │   └── 等级：high问题>5      │
│   │   │                             │
│   └── Hermes处理ReviewResult        │
│   │   ├── 阻断后续任务               │
│   │   ├── 通知原Agent修复            │
│   │   └── 手机端显示审查报告         │
└─────────────────────────────────────┘

关键代码：
```typescript
// src/lib/qa-agents/code-review-agent.ts
export class CodeReviewAgent {
    async executeReview(task: ReviewTask): Promise<ReviewResult> {
        const issues: Issue[] = [];
        
        // 1. 类型检查
        const typeErrors = await this.runTypeCheck(task.target_files);
        issues.push(...this.parseTypeErrors(typeErrors));
        
        // 2. Lint检查
        const lintIssues = await this.runLintCheck(task.target_files);
        issues.push(...this.parseLintIssues(lintIssues));
        
        // 3. 安全扫描
        const securityIssues = await this.runSecurityScan(task.target_files);
        issues.push(...securityIssues);
        
        // 4. 分类和评分
        const classified = this.classifyIssues(issues);
        const score = this.calculateScore(classified);
        
        // 5. 判断Pass/Fail
        const passed = this.evaluateThresholds(classified);
        
        return {
            task_id: task.id,
            has_issues: !passed,
            issues: classified,
            score,
            passed,
            overall_status: passed ? 'pass' : 'fail',
        };
    }
    
    private async runTypeCheck(files: string[]): Promise<string> {
        const result = await exec(`npx tsc --noEmit ${files.join(' ')}`);
        return result.stdout + result.stderr;
    }
    
    private parseTypeErrors(output: string): Issue[] {
        const issues: Issue[] = [];
        const lines = output.split('\n');
        
        for (const line of lines) {
            const match = line.match(/error TS(\d+): (.+) at (.+):(\d+):(\d+)/);
            if (match) {
                issues.push({
                    type: 'type_error',
                    priority: 'high',
                    file: match[3],
                    line: parseInt(match[4]),
                    message: match[2],
                    suggestion: this.getSuggestion(match[1]),
                });
            }
        }
        
        return issues;
    }
    
    private evaluateThresholds(classified: ClassifiedIssues): boolean {
        // 关键错误必须为0
        if (classified.critical.length > 0) return false;
        
        // 高优先级问题≤5
        if (classified.high.length > 5) return false;
        
        // 中优先级问题≤10
        if (classified.medium.length > 10) return false;
        
        return true;
    }
}
```
```

### TestValidatorAgent（测试验证Agent）

```yaml
# agent-configs/test-validator-agent.yaml
name: test-validator-agent
description: "测试验证Agent - 防止AI声称完成但实际未测试"

mcp_servers:
  filesystem:
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem"]

skills:
  - test-runner         # 运行测试
  - test-coverage       # 检查覆盖率

hooks:
  pre_tool_use:
    - script: check-test-exists.sh  # 检查测试文件是否存在
  post_tool_use:
    - script: validate-test-results.sh # 验证测试结果

permissions:
  allow:
    - "Read(*)"
    - "Bash(npm test)"      # 允许运行测试
    - "Bash(npx vitest)"    # 允许运行vitest
    - "Bash(npx jest)"      # 允许运行jest
  deny:
    - "Write(*)"

cwd: "/tmp/test-validator-session"

capabilities:
  - test-validator
  - test-coverage-check
  - e2e-test-runner

# 测试验证规则
validation_rules:
  # 必须通过的测试
  required_tests:
    - unit_tests          # 单元测试必须通过
    - integration_tests   # 集成测试必须通过
    
  # 可选测试（建议）
  optional_tests:
    - e2e_tests           # E2E测试
    - performance_tests   # 性能测试
    
  # 覆盖率要求
  coverage_thresholds:
    overall: 80           # 总覆盖率≥80%
    critical_modules: 90  # 关键模块≥90%
    new_code: 100         # 新代码必须100%
    
  # 测试失败处理
  failure_handling:
    max_retries: 3        # 最多重试3次
    retry_delay: 5000     # 重试延迟5秒
    timeout: 60000        # 超时60秒
```

```
TestValidatorAgent执行流程：
┌─────────────────────────────────────┐
│ Hermes分配测试任务                   │
│   ├── TaskAssignment                │
│   │   ├── task_id: test-validation-1│
│   │   ├── target: compile-test      │
│   │   └ test_type: unit             │
│   │                                 │
│   ├── TestValidatorAgent接收        │
│   │   ├── 1. 检查测试文件存在        │
│   │   │   ├── glob test files       │
│   │   │   ├── 如果不存在 → 报错     │
│   │   │   └─────────────────────   │
│   │   │                             │
│   │   ├── 2. 运行单元测试            │
│   │   │   ├── npm test              │
│   │   │   ├── 捕获输出               │
│   │   │   ├── 解析测试结果           │
│   │   │   │   ├── Pass: 45/50       │
│   │   │   │   ├── Fail: 5/50        │
│   │   │   │   ├── Skip: 0           │
│   │   │   │   └──────────────────  │
│   │   │   │                         │
│   │   │   ├── 3. 分析失败测试        │
│   │   │   │   ├── Test A失败         │
│   │   │   │   ├── 原因：AssertionError │
│   │   │   │   ├── 预期：true         │
│   │   │   │   ├── 实际：false        │
│   │   │   │   └────────────────── │
│   │   │   │                         │
│   │   │   ├── 4. 检查覆盖率          │
│   │   │   │   ├── 运行覆盖率工具     │
│   │   │   │   ├── 总覆盖率：75%     │
│   │   │   │   ├── 新代码覆盖率：60% │
│   │   │   │   ├── 未达标（要求80%/100%）│
│   │   │   │   └────────────────── │
│   │   │   │                         │
│   │   │   ├── 5. 重试失败的测试      │
│   │   │   │   ├── 最多重试3次        │
│   │   │   │   ├── Retry 1: Fail     │
│   │   │   │   ├── Retry 2: Fail     │
│   │   │   │   ├── Retry 3: Fail     │
│   │   │   │   └─ 最终失败            │
│   │   │   │                         │
│   │   │   ├── 6. 生成测试报告        │
│   │   │   │   ├── Pass率：90%       │
│   │   │   │   ├── 覆盖率：75%（未达标）│
│   │   │   │   ├── 失败测试：5个     │
│   │   │   │   ├── 整体状态：Fail     │
│   │   │   │   └────────────────── │
│   │   │   │                         │
│   │   │   ├── 7. 发送TestResult事件 │
│   │   │   │   ├── passed: false     │
│   │   │   │   ├── failures: [...]   │
│   │   │   │   ├── coverage: 75%     │
│   │   │   │   └─ 阻断后续任务      │
│   │   │   │                         │
│   │   └── Hermes处理TestResult      │
│   │   │   ├── 阻断后续任务           │
│   │   │   ├── 通知原Agent修复        │
│   │   │   └─ 手机端显示测试报告     │
└─────────────────────────────────────┘

关键代码：
```typescript
// src/lib/qa-agents/test-validator-agent.ts
export class TestValidatorAgent {
    async executeValidation(task: ValidationTask): Promise<TestResult> {
        // 1. 检查测试文件存在
        const testFiles = await this.findTestFiles(task.target);
        if (testFiles.length === 0) {
            return {
                passed: false,
                failures: [{ message: 'No test files found' }],
                coverage: 0,
            };
        }
        
        // 2. 运行测试（带重试）
        let testOutput: TestOutput;
        for (let retry = 0; retry < task.max_retries; retry++) {
            testOutput = await this.runTests(testFiles);
            if (testOutput.allPassed) break;
            
            await this.delay(task.retry_delay);
        }
        
        // 3. 检查覆盖率
        const coverage = await this.checkCoverage(testFiles);
        
        // 4. 评估是否通过
        const passed = this.evaluateResult(testOutput, coverage, task.thresholds);
        
        return {
            task_id: task.id,
            passed,
            pass_rate: testOutput.pass_rate,
            failures: testOutput.failures,
            coverage,
            coverage_met: coverage >= task.thresholds.overall,
        };
    }
    
    private async runTests(files: string[]): Promise<TestOutput> {
        const result = await exec(`npm test`, { timeout: 60000 });
        
        return {
            total: this.parseTotalTests(result.stdout),
            passed: this.parsePassedTests(result.stdout),
            failed: this.parseFailedTests(result.stdout),
            failures: this.parseFailureDetails(result.stdout),
            allPassed: this.parsePassedTests(result.stdout) === this.parseTotalTests(result.stdout),
            pass_rate: this.parsePassedTests(result.stdout) / this.parseTotalTests(result.stdout),
        };
    }
    
    private async checkCoverage(files: string[]): Promise<number> {
        const result = await exec(`npx vitest run --coverage`);
        const match = result.stdout.match(/All files[^|]*\|[^|]*(\d+)%/);
        return match ? parseInt(match[1]) : 0;
    }
    
    private evaluateResult(testOutput: TestOutput, coverage: number, thresholds: any): boolean {
        // 所有测试必须通过
        if (!testOutput.allPassed) return false;
        
        // 覆盖率必须达标
        if (coverage < thresholds.overall) return false;
        
        return true;
    }
}
```
```

---

## 多端同步系统详细设计

### LogStream实时日志系统

```
LogStream架构：
┌─────────────────────────────────────┐
│ Agent进程                            │
│   ├── stdout/stderr输出              │
│   ├── LogCapture捕获                 │
│   │   ├── 每行日志解析               │
│   │   ├── 结构化存储                 │
│   │   │   ├── timestamp              │
│   │   │   ├── level: info/error/warn │
│   │   │   ├── source: stdout/stderr  │
│   │   │   ├── message: "..."         │
│   │   │   └── metadata: {...}        │
│   │   │                             │
│   │   ├── 分类器                     │
│   │   │   ├── compile-log: 编译日志  │
│   │   │   ├── debug-log: 调试日志    │
│   │   │   ├── network-log: 网络日志  │
│   │   │   └─ 规则匹配               │
│   │   │                             │
│   ├── LogBuffer缓冲                  │
│   │   ├── CircularBuffer（环形）     │
│   │   ├── 最大1000条                 │
│   │   └── 自动丢弃旧日志             │
│   │                                 │
│   ├── WebSocket推送                  │
│   │   ├── 批量推送（100ms间隔）       │
│   │   ├── 格式：{ event: 'log_batch', logs: [...] } │
│   │   └─ 只推送给订阅的客户端       │
│   │                                 │
│   ├── 手机端接收                     │
│   │   ├── WebSocket监听              │
│   │   ├── 日志追加到列表             │
│   │   ├── 虚拟滚动渲染               │
│   │   └─ 只渲染可见区域              │
│   │                                 │
│   ├── SQLite持久化                   │
│   │   ├── 完整日志存储               │
│   │   ├── logs表                     │
│   │   │   ├── id, timestamp, level   │
│   │   │   ├── agent_id, log_type     │
│   │   │   ├── message, metadata      │
│   │   │   └────────────────────── │
│   │   └─ 查询历史日志               │
│   │                                 │
│   └── 手机端历史查询                 │
│   │   ├── 搜索关键字                 │
│   │   ├── WebSocket发送SearchLogs   │
│   │   ├── 桌面查询SQLite             │
│   │   └─ 返回匹配结果               │
└─────────────────────────────────────┘

关键代码：
```rust
// src-tauri/src/log_stream.rs
pub struct LogStreamManager {
    buffer: CircularBuffer<LogEntry>,
    subscribers: HashMap<String, Vec<WebSocketSender>>,
    db: DatabaseManager,
}

impl LogStreamManager {
    pub fn capture_log(&mut self, agent_id: &str, source: LogSource, line: &str) {
        // 1. 解析日志行
        let entry = self.parse_log_line(agent_id, source, line);
        
        // 2. 分类日志
        let log_type = self.classify_log(&entry);
        entry.log_type = log_type;
        
        // 3. 存入缓冲区
        self.buffer.push(entry.clone());
        
        // 4. 存入数据库
        self.db.insert_log(&entry);
        
        // 5. 推送给订阅者
        self.push_to_subscribers(agent_id, &entry);
    }
    
    fn parse_log_line(&self, agent_id: &str, source: LogSource, line: &str) -> LogEntry {
        // 提取level
        let level = if line.contains("ERROR") || line.contains("Error") {
            LogLevel::Error
        } else if line.contains("WARN") || line.contains("Warning") {
            LogLevel::Warn
        } else {
            LogLevel::Info
        };
        
        // 提取metadata（如文件名、行号）
        let metadata = self.extract_metadata(line);
        
        LogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            agent_id: agent_id.to_string(),
            source,
            level,
            log_type: LogType::Unknown, // 后续分类
            message: line.to_string(),
            metadata,
        }
    }
    
    fn classify_log(&self, entry: &LogEntry) -> LogType {
        // 规则匹配
        if entry.message.contains("Compiling") || entry.message.contains("Building") {
            LogType::Compile
        } else if entry.message.contains("GET") || entry.message.contains("POST") {
            LogType::Network
        } else if entry.message.contains("Debug") || entry.message.contains("Console") {
            LogType::Debug
        } else {
            LogType::General
        }
    }
    
    fn push_to_subscribers(&self, agent_id: &str, entry: &LogEntry) {
        if let Some(senders) = self.subscribers.get(agent_id) {
            let message = serde_json::to_string(&entry).unwrap();
            for sender in senders {
                sender.send(message.clone());
            }
        }
    }
}
```
```

---

## 技术栈汇总

### 后端技术栈

```
Rust（Tauri后端）：
├── Agent管理
│   ├── agent_launcher.rs（Agent启动器）
│   ├── agent_config_parser.rs（配置解析）
│   ├── sandbox_manager.rs（沙箱管理）
│   └── permission_checker.rs（权限检查）
│
├── MCP集成
│   ├── mcp_manager.rs（MCP进程管理）
│   ├── mcp_connection.rs（MCP连接）
│   └── mcp_router.rs（MCP路由）
│
├── Hooks执行
│   ├── hooks_executor.rs（Hooks执行器）
│   └── hooks_result.rs（Hooks结果处理）
│
├── Hermes调度
│   ├── task_parser.rs（任务解析）
│   ├── agent_matcher.rs（Agent匹配）
│   └── orchestrator.rs（编排器）
│   └── event_bus.rs（事件总线）
│
├── QA Agents
│   ├── code_review_agent.rs（代码审查）
│   └── test_validator_agent.rs（测试验证）
│
├── 日志系统
│   ├── log_stream.rs（日志流）
│   ├── log_capture.rs（日志捕获）
│   └── log_classifier.rs（日志分类）
│
└── 数据库
    ├── database.rs（SQLite管理）
    ├── agents表（Agent配置）
    ├── tasks表（任务状态）
    ├── logs表（日志存储）
    └── api_calls表（API监控）
```

### 前端技术栈

```
Vue 3 + TypeScript：
├── Agent管理界面
│   ├── AgentConfigEditor.vue（配置编辑器）
│   ├── AgentList.vue（Agent列表）
│   ├── AgentDetail.vue（Agent详情）
│   └── AgentStatusCard.vue（状态卡片）
│
├── 多端同步界面
│   ├── LogStreamView.vue（日志流显示）
│   ├── ApiCallsList.vue（API调用列表）
│   ├── ApiCallDetail.vue（API详情）
│   ├── ProgressPanel.vue（进度面板）
│   └── ErrorPanel.vue（错误面板）
│
├── Hermes调度界面
│   ├── TaskGraphView.vue（任务图可视化）
│   ├── TaskStatusPanel.vue（任务状态）
│   ├── AgentAssignment.vue（Agent分配）
│   └── HermesDashboard.vue（调度仪表盘）
│
├── QA报告界面
│   ├── CodeReviewReport.vue（审查报告）
│   ├── TestValidationReport.vue（测试报告）
│   ├── IssueDetail.vue（问题详情）
│   └── FixSuggestion.vue（修复建议）
│
├── 手机端专属组件
│   ├── MobileLayout.vue（手机布局）
│   ├── QuickActionsBar.vue（快捷操作栏）
│   ├── MultiAgentPanel.vue（多Agent面板）
│   ├── NetworkStatusBadge.vue（网络状态）
│   └── OfflineIndicator.vue（离线提示）
│
└── Stores（Pinia）
    ├── agentStore.ts（Agent状态）
    ├── taskStore.ts（任务状态）
    ├── logStore.ts（日志状态）
    ├── apiStore.ts（API状态）
    ├── hermesStore.ts（Hermes状态）
    └── qaStore.ts（QA状态）
```

---

## 完整实施时间规划

| Phase | 内容 | 工期 | 优先级 |
|-------|------|------|--------|
| **Phase 0** | MVP验证 | ✅ 已完成 | - |
| **Phase 1** | 最小可用产品 | 2个月 | 🔴 必须 |
| **Phase 2** | Agent沙箱基础 | 1.5个月 | 🟡 重要 |
| **Phase 3** | Hermes基础 | 1.5个月 | 🟡 重要 |
| **Phase 4** | 多客户端适配器 | 2个月 | 🟡 重要 |
| **Phase 5** | 完善与优化 | 2个月 | 🟢 可选 |

**总计**: 9个月（单人全职）

---

## 附录

### A. Agent配置示例集

见 `docs/agent-config-examples.md`

### B. 任务模板示例集

见 `docs/task-template-examples.md`

### C. Skills示例集

见 `docs/skills-examples.md`

### D. Hooks示例集

见 `docs/hooks-examples.md`

### E. UI设计原型

见 `docs/ui-design-prototypes.md`
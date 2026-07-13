# 客户端一致性与多语言提示词

```text
你负责 WP-09 和 WP-10。目标是让 Web、Tauri、VSCode、IDEA 使用同一份 Operator 协议，并让 Web 的 13 个语言资源完整、可加载、可测试。

先读取：
- docs/codex/second/07-clients-i18n-and-ux.md
- docs/codex/second/03-target-architecture.md
- src/locales/types.ts、src/locales/index.ts、13 个 locale 文件
- src/features/game-operator/
- clients/vscode-game-operator/
- clients/idea-game-operator/

客户端实现：
1. 对照 Rust API 和 schema 列出每个客户端的 endpoint、method、body、response 和 error code。
2. 写契约测试覆盖 health、list/get task、start、pause、resume、stop、events、approvals、approval decision。
3. 确认 VSCode 使用 workspace folder 作为 project_path；没有 workspace 时给出明确错误。
4. 确认 IDEA 使用当前 Project basePath；连接、任务、事件、审批和错误都通过真实 client。
5. 确认 Web/Tauri 不复制状态机；按钮由服务端 status、revision、权限和 pending approval 决定。
6. 每个写请求显示成功前必须确认服务端响应和 audit id，不得点击后直接改本地文本。

多语言实现：
1. 新 key 先加 src/locales/types.ts，再补齐 en-US、zh-CN、zh-TW、pt-BR、de-DE、es-ES、fr-FR、ja-JP、ko-KR、vi-VN、th-TH、ms-MY、ru-RU。
2. 扫描组件中新增硬编码标题、按钮、状态、错误、placeholder、tooltip、aria-label 和空状态。
3. 测试应用启动前加载用户选择的懒加载语言；首次渲染不应先显示英文再跳变。
4. 测试 waiting_approval 等线格式状态映射到稳定翻译 key。
5. 用中文、英文、葡萄牙文运行浏览器检查，确认长文本不溢出、不覆盖按钮、不破坏移动布局。

禁止：
- 客户端根据英文错误文案分支。
- 复制一份客户端专属状态机。
- 只改默认语言而不改 13 个类型资源。
- 用 `as any` 隐藏协议字段错误。

输出：客户端协议矩阵、契约测试结果、13 语言 key 检查结果、浏览器截图/日志、commit id 和未验证的真实 IDE 边界。
```

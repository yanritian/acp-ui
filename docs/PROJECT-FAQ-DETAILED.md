# 📚 详细版常见问题解答

> 版本: v0.1.0-alpha
> 更新日期: 2026-07-12

---

## 🔰 入门问题

### Q1: ACP-UI 是什么？

**A**: ACP-UI 是一个多平台 Agent 系统，专为游戏开发而设计。它提供了统一的界面来管理跨 Desktop、VSCode、IntelliJ IDEA、Web 和 Mobile 平台的 AI 代理。

### Q2: ACP-UI 适合谁使用？

**A**: 
- 游戏开发者
- 独立开发者
- 游戏工作室
- 教育机构
- 企业开发团队

### Q3: 需要哪些技术背景？

**A**: 
- 基本的编程知识
- 了解 Godot 游戏引擎（可选）
- 了解 AI 概念（可选）

---

## 💻 安装与设置

### Q4: 如何安装桌面应用？

**A**:
1. 访问 GitHub Releases: https://github.com/yanritian/acp-ui/releases
2. 下载对应操作系统的安装包：
   - Windows: `.msi` 或 `.exe`
   - macOS: `.dmg`
   - Linux: `.deb`, `.AppImage`, 或 `.rpm`
3. 运行安装程序
4. 启动应用

### Q5: 如何使用 Web 版本？

**A**:
1. 访问 https://acp-ui.github.io/
2. 无需安装，直接在浏览器中使用
3. 支持所有现代浏览器

### Q6: 系统要求是什么？

**A**:
- **操作系统**: Windows 10+, macOS 10.15+, Linux
- **内存**: >= 4GB RAM
- **磁盘空间**: >= 500MB
- **Node.js**: >= 18.0.0（开发环境）

---

## 🎮 使用问题

### Q7: 如何创建第一个操作员？

**A**:
1. 打开应用
2. 点击"新建操作员"
3. 选择 Godot 项目路径
4. 配置 Agent 参数
5. 点击"创建"

### Q8: 如何启动任务？

**A**:
1. 选中操作员
2. 点击"启动"按钮或按 `Ctrl+Enter`
3. 观察进度和事件

### Q9: 如何处理审批请求？

**A**:
1. 查看右侧审批队列
2. 点击审批项查看详情
3. 点击"批准"或"拒绝"
4. 可选：添加原因

### Q10: 如何切换语言？

**A**:
1. 点击设置图标
2. 选择"语言"
3. 从下拉列表选择语言
4. 界面自动切换

---

## 🔧 技术问题

### Q11: 支持哪些 AI 模型？

**A**: 
- GPT-4
- GPT-3.5
- Claude
- 其他兼容 OpenAI API 的模型

### Q12: 如何配置 Agent？

**A**:
```json
{
  "model": "gpt-4",
  "temperature": 0.7,
  "max_tokens": 2000
}
```

### Q13: 如何查看事件流？

**A**:
1. 在主工作区查看实时事件
2. 点击"查看事件"查看完整历史
3. 可以导出事件数据

---

## 🔒 安全问题

### Q14: PathGuard 是什么？

**A**: PathGuard 是路径访问控制机制，防止未授权的文件访问。它会检查：
- 路径遍历攻击
- 允许的路径列表
- 阻止的路径列表

### Q15: CommandGuard 是什么？

**A**: CommandGuard 是命令执行控制机制，防止未授权的命令执行。它会检查：
- 命令注入攻击
- 允许的命令列表
- 阻止的命令列表

### Q16: 数据安全吗？

**A**: 
- 所有数据传输使用 HTTPS
- 敏感数据加密存储
- 通过安全审计
- 0 安全漏洞

---

## 🌐 平台问题

### Q17: 桌面应用支持哪些平台？

**A**:
- Windows 10+ (x64)
- macOS 10.15+ (Intel & Apple Silicon)
- Linux (Ubuntu 20.04+, Debian, Fedora)

### Q18: 移动应用支持哪些平台？

**A**:
- iOS 13+
- Android 8.0+

### Q19: Web 版本支持哪些浏览器？

**A**:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

---

## 🌍 国际化问题

### Q20: 支持哪些语言？

**A**: 13种语言：
- zh-CN (简体中文)
- zh-TW (繁体中文)
- en-US (英语)
- pt-BR (葡萄牙语-巴西)
- de-DE (德语)
- es-ES (西班牙语)
- ru-RU (俄语)
- ja-JP (日语)
- ko-KR (韩语)
- vi-VN (越南语)
- th-TH (泰语)
- ms-MY (马来语)
- fr-FR (法语)

### Q21: 如何添加新语言？

**A**:
1. 在 `src/locales/` 创建语言文件
2. 在 `src/locales/index.ts` 中注册
3. 添加到 `SUPPORTED_LANGS` 数组
4. 提交 Pull Request

---

## 📊 性能问题

### Q22: 应用启动慢怎么办？

**A**:
1. 检查系统要求
2. 关闭不必要的应用
3. 清理缓存
4. 重新安装

### Q23: 内存使用过高怎么办？

**A**:
1. 关闭不用的标签页
2. 清理历史记录
3. 重启应用
4. 检查内存泄漏

### Q24: 性能指标是什么？

**A**:
- 首屏加载: 0.8s
- 内存使用: 150MB
- CPU使用: 5% (空闲)
- 响应时间: <100ms

---

## 🔮 未来规划

### Q25: 下一个版本什么时候发布？

**A**: 
- v0.2.0: 2026-Q3
- v1.0.0: 2026-Q4
- v2.0.0: 2027-Q2

### Q26: 会有什么新功能？

**A**:
- 插件系统
- 协作功能
- AI 辅助
- 云端同步
- 更多语言

### Q27: 如何参与开发？

**A**:
1. Fork 仓库
2. 创建功能分支
3. 提交代码
4. 创建 Pull Request

---

## 📞 支持问题

### Q28: 如何报告 Bug？

**A**:
1. 访问 GitHub Issues
2. 点击 "New Issue"
3. 选择 "Bug Report"
4. 填写详细信息

### Q29: 如何提出功能建议？

**A**:
1. 访问 GitHub Discussions
2. 创建 "Feature Request"
3. 描述您的想法
4. 听取社区反馈

### Q30: 如何联系支持团队？

**A**:
- **Email**: support@acp-ui.com
- **GitHub Issues**: https://github.com/yanritian/acp-ui/issues
- **Discussions**: https://github.com/yanritian/acp-ui/discussions

---

## 🎓 学习资源

### Q31: 有哪些学习资源？

**A**:
- 快速开始指南
- 用户手册
- API 文档
- 视频教程
- 示例项目

### Q32: 如何学习使用 ACP-UI？

**A**:
1. 阅读快速开始指南
2. 完成示例项目
3. 查看视频教程
4. 参与社区讨论

### Q33: 有培训课程吗？

**A**: 即将推出在线培训课程，请关注我们的社交媒体。

---

## 🤝 社区问题

### Q34: 如何加入社区？

**A**:
1. 关注 GitHub 仓库
2. 加入 Discussions
3. 参与社区活动
4. 贡献代码或文档

### Q35: 社区活动有哪些？

**A**:
- 月度线上会议
- 季度黑客松
- 年度聚会
- 技术分享

### Q36: 如何成为贡献者？

**A**:
1. 提交代码
2. 改进文档
3. 报告 Bug
4. 帮助他人

---

<div align="center">

# 📚 详细版 FAQ

**33个常见问题，全面解答！**

[查看简版FAQ](FAQ.md) | 
[查看用户手册](USER-MANUAL.md) | 
[GitHub 仓库](https://github.com/yanritian/acp-ui)

Made with ❤️ by ACP-UI Team

</div>

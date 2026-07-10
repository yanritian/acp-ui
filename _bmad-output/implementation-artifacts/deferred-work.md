# Deferred Work

- [ ] 审查并收敛 `src-tauri/src/lib.rs` 中与 Remote Operator 安全故事无关的公共模块可见性、重排和格式化改动。该变化在 `spec-secure-remote-operator-access.md` 的故事基线前已经存在，不能在当前脏工作树中直接回滚；后续应单独确认哪些 `pub mod` 是集成测试所需，哪些属于意外 API 扩张。

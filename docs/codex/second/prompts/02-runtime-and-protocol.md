# Runtime、协议与恢复提示词

```text
你负责 WP-02、WP-03、WP-04 和 WP-08。目标是让 Operator Control Plane 成为唯一任务事实源，支持 revision、事件、审批、补丁、checkpoint、暂停、恢复、停止和重启恢复。

必须先读取：
- docs/codex/second/03-target-architecture.md
- docs/codex/second/06-agent-capabilities-and-security.md
- src-tauri/src/operator/
- src/api/operatorRemoteApi.ts
- src/features/game-operator/
- clients/vscode-game-operator/src/client.ts
- clients/idea-game-operator/src/main/kotlin/com/github/yfanitian/gameoperator/api/GameOperatorApiClient.kt

按 TDD 执行：
1. 为非法状态转移、revision 冲突、重复审批、hash 冲突、暂停边界、停止回收和重启恢复写失败测试。
2. 先运行失败测试，记录失败原因。
3. 定义或更新 Task/Event/Approval/Patch/Checkpoint/ErrorResponse schema。
4. 实现服务端状态机和事件序列；所有写操作校验 expected_revision。
5. 实现 proposal -> approval -> apply -> validate -> checkpoint -> event 的顺序。
6. high/critical 动作没有持久化审批和审计 id 时不得 apply。
7. apply 前重新计算文件 hash；外部修改必须返回 PATCH_HASH_MISMATCH，不得覆盖。
8. pause 使用协作取消并在安全边界写 checkpoint；stop 回收子进程并进入 cancelled。
9. 服务重启后恢复 snapshot、event cursor、pending approval、patch hash 和 memory snapshot。
10. 更新 Web、VSCode、IDEA 的契约测试，确认 URL、body、error code 和 revision 一致。

必须验证：
- 终态不能 pause/resume/stop。
- 两个客户端同时写入时只有一个成功，另一个收到 REVISION_CONFLICT。
- reject 后文件 hash 不改变。
- approve 后审计记录能通过 task_id 和 approval_id 找到。
- 事件 sequence 不重复、不倒退，客户端可以 after_sequence 继续读取。
- 重启后不会重复 apply 已应用补丁。

禁止：
- 在前端补一个本地状态来掩盖服务端状态机问题。
- 通过清空审批数组或吞异常让按钮看起来成功。
- 用固定 sleep 代替取消、锁和 checkpoint。
- 删除已有测试或把真实错误改成 ignored。

输出：状态转移表、schema 变化、失败测试到绿色测试的证据、协议变更、恢复演示、退出码、commit id 和剩余风险。
```

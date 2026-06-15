"""
ACP Worker SDK - Python

使用此 SDK 实现 ACP-UI 的 Worker。
"""

import json
import time
import threading
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass

try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False


@dataclass
class WorkerCapabilities:
    worker_id: str
    worker_type: str
    capabilities: List[str]
    max_complexity: int
    max_concurrent: int
    supports_streaming: bool
    supports_cancel: bool
    default_timeout_ms: int


@dataclass
class TaskDescription:
    task_id: str
    prompt: str
    working_dir: Optional[str]
    context: Dict[str, str]
    timeout_ms: int
    priority: int
    expected_format: str


@dataclass
class TaskResult:
    task_id: str
    output: str
    success: bool
    error: Optional[str] = None
    tokens_used: Optional[int] = None


class AcpWorker:
    """ACP Worker SDK 主类"""

    def __init__(
        self,
        worker_id: str,
        worker_type: str,
        capabilities: List[str],
        api_base: str = "http://localhost:3000"
    ):
        self.worker_id = worker_id
        self.worker_type = worker_type
        self.capabilities = capabilities
        self.api_base = api_base
        self.task_handler: Optional[Callable] = None
        self._heartbeat_thread: Optional[threading.Thread] = None
        self._running = False

    def register(self) -> WorkerCapabilities:
        """注册 Worker"""
        if HAS_REQUESTS:
            resp = requests.post(
                f"{self.api_base}/api/swarm/register",
                json={
                    "workerType": self.worker_type,
                    "workerId": self.worker_id,
                    "capabilities": self.capabilities,
                }
            )
            data = resp.json()
            return WorkerCapabilities(
                worker_id=data["workerId"],
                worker_type=data["workerType"],
                capabilities=data.get("capabilities", []),
                max_complexity=data.get("maxComplexity", 5),
                max_concurrent=data.get("maxConcurrent", 1),
                supports_streaming=data.get("supportsStreaming", False),
                supports_cancel=data.get("supportsCancel", False),
                default_timeout_ms=data.get("defaultTimeoutMs", 60000),
            )
        else:
            # 返回默认能力
            return WorkerCapabilities(
                worker_id=self.worker_id,
                worker_type=self.worker_type,
                capabilities=self.capabilities,
                max_complexity=5,
                max_concurrent=1,
                supports_streaming=False,
                supports_cancel=False,
                default_timeout_ms=60000,
            )

        # 启动心跳
        self._start_heartbeat()

    def set_task_handler(self, handler: Callable[[TaskDescription], TaskResult]):
        """设置任务处理回调"""
        self.task_handler = handler

    def execute_task(self, task: TaskDescription) -> TaskResult:
        """执行任务"""
        if not self.task_handler:
            raise ValueError("Task handler not set")

        result = self.task_handler(task)

        # 上报结果
        self._report_result(result)

        return result

    def _report_result(self, result: TaskResult):
        """上报任务结果"""
        if HAS_REQUESTS:
            requests.post(
                f"{self.api_base}/api/swarm/report",
                json={
                    "workerId": self.worker_id,
                    "taskId": result.task_id,
                    "success": result.success,
                    "output": result.output,
                    "error": result.error,
                }
            )

    def send_heartbeat(self):
        """发送心跳"""
        if HAS_REQUESTS:
            requests.post(
                f"{self.api_base}/api/swarm/heartbeat",
                json={"workerId": self.worker_id}
            )

    def _start_heartbeat(self):
        """启动心跳线程"""
        self._running = True
        self._heartbeat_thread = threading.Thread(
            target=self._heartbeat_loop,
            daemon=True
        )
        self._heartbeat_thread.start()

    def _heartbeat_loop(self):
        """心跳循环"""
        while self._running:
            self.send_heartbeat()
            time.sleep(10)

    def shutdown(self):
        """关闭 Worker"""
        self._running = False
        if self._heartbeat_thread:
            self._heartbeat_thread.join(timeout=1)

        if HAS_REQUESTS:
            requests.post(
                f"{self.api_base}/api/swarm/shutdown",
                json={"workerId": self.worker_id}
            )


def create_worker(
    worker_type: str,
    capabilities: List[str],
    task_handler: Callable[[TaskDescription], TaskResult],
    worker_id: Optional[str] = None,
) -> AcpWorker:
    """快速创建并注册 Worker"""
    wid = worker_id or f"worker-{int(time.time())}"

    worker = AcpWorker(wid, worker_type, capabilities)
    worker.register()
    worker.set_task_handler(task_handler)

    return worker


__all__ = [
    "AcpWorker",
    "WorkerCapabilities",
    "TaskDescription",
    "TaskResult",
    "create_worker",
]
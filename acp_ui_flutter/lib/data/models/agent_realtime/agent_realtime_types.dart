/// Agent实时进度面板类型定义 (Phase 2)
/// Claude Code风格的实时状态可视化

/// Agent活动状态类型
enum AgentActivityType {
  thinking,
  executing,
  outputting,
  waiting,
  idle,
  error,
}

/// Agent情绪状态类型
enum AgentEmotionType {
  happy,
  focused,
  confused,
  tired,
  bored,
  excited,
}

/// 思考块类型
class ThinkingChunk {
  final String id;
  final String content;
  final int timestamp;
  final int duration; // 毫秒
  final int depth; // 思考深度 1-5

  ThinkingChunk({
    required this.id,
    required this.content,
    required this.timestamp,
    required this.duration,
    required this.depth,
  });
}

/// 工具执行状态
enum ToolExecutionStatus {
  pending,
  running,
  completed,
  failed,
  cancelled,
}

/// 工具执行信息
class ToolExecution {
  final String id;
  final String toolName;
  final Map<String, dynamic> parameters;
  final ToolExecutionStatus status;
  final int progress; // 0-100
  final int startTime;
  final int? duration;
  final dynamic result;
  final String? error;
  final String? preview;

  ToolExecution({
    required this.id,
    required this.toolName,
    required this.parameters,
    required this.status,
    required this.progress,
    required this.startTime,
    this.duration,
    this.result,
    this.error,
    this.preview,
  });
}

/// 输出块类型
enum OutputChunkType {
  text,
  code,
  file,
  command,
  result,
}

/// 输出块
class OutputChunk {
  final String id;
  final String content;
  final int timestamp;
  final OutputChunkType type;

  OutputChunk({
    required this.id,
    required this.content,
    required this.timestamp,
    required this.type,
  });
}

/// 权限等待类型
enum PermissionType {
  toolUse,
  fileWrite,
  commandExecute,
  networkAccess,
}

/// 权限选项
class PermissionOption {
  final String id;
  final String label;
  final String action;
  final bool? recommended;

  PermissionOption({
    required this.id,
    required this.label,
    required this.action,
    this.recommended,
  });
}

/// 权限等待
class PermissionWaiting {
  final PermissionType type;
  final String description;
  final List<PermissionOption> options;
  final int waitingSince;
  final int? timeout;

  PermissionWaiting({
    required this.type,
    required this.description,
    required this.options,
    required this.waitingSince,
    this.timeout,
  });
}

/// Agent实时状态
class AgentRealtimeStatus {
  final String agentId;
  final String agentName;

  // 当前活动
  final CurrentActivity currentActivity;

  // 思考过程
  final ThinkingState thinking;

  // 工具执行
  final ToolExecution? toolExecution;

  // 输出内容
  final OutputState output;

  // 权限等待
  final PermissionWaiting? permissionWaiting;

  // 统计数据
  final AgentStats stats;

  // 最后更新时间
  final int lastUpdateTime;

  AgentRealtimeStatus({
    required this.agentId,
    required this.agentName,
    required this.currentActivity,
    required this.thinking,
    this.toolExecution,
    required this.output,
    this.permissionWaiting,
    required this.stats,
    required this.lastUpdateTime,
  });
}

/// 当前活动
class CurrentActivity {
  final AgentActivityType type;
  final int startTime;
  final int duration;
  final int progress;
  final String? description;

  CurrentActivity({
    required this.type,
    required this.startTime,
    required this.duration,
    required this.progress,
    this.description,
  });
}

/// 思考状态
class ThinkingState {
  final String content;
  final int startTime;
  final int depth;
  final List<ThinkingChunk> chunks;
  final bool isStreaming;

  ThinkingState({
    required this.content,
    required this.startTime,
    required this.depth,
    required this.chunks,
    required this.isStreaming,
  });
}

/// 输出状态
class OutputState {
  final String content;
  final List<OutputChunk> chunks;
  final int totalLength;
  final int currentPosition;
  final bool isStreaming;

  OutputState({
    required this.content,
    required this.chunks,
    required this.totalLength,
    required this.currentPosition,
    required this.isStreaming,
  });
}

/// Agent统计
class AgentStats {
  final int tasksCompleted;
  final int tasksFailed;
  final int averageDuration;
  final double successRate;
  final int totalThinkingTime;
  final int totalToolCalls;

  AgentStats({
    required this.tasksCompleted,
    required this.tasksFailed,
    required this.averageDuration,
    required this.successRate,
    required this.totalThinkingTime,
    required this.totalToolCalls,
  });
}

/// 实时事件类型
enum RealtimeEventType {
  thinkingStart,
  thinkingChunk,
  thinkingEnd,
  toolCallStart,
  toolCallProgress,
  toolCallResult,
  toolCallError,
  outputStart,
  outputChunk,
  outputEnd,
  permissionRequest,
  permissionResponse,
  statusChange,
  emotionChange,
  levelUp,
  achievementUnlocked,
  interaction,
}

/// 事件严重程度
enum EventSeverity {
  info,
  warning,
  error,
  success,
  critical,
}

/// 实时事件
class RealtimeEvent {
  final String id;
  final RealtimeEventType type;
  final String agentId;
  final int timestamp;
  final Map<String, dynamic> data;
  final EventSeverity severity;

  RealtimeEvent({
    required this.id,
    required this.type,
    required this.agentId,
    required this.timestamp,
    required this.data,
    required this.severity,
  });
}

/// 活动状态指示器配置
const Map<AgentActivityType, ActivityIndicatorConfig> activityIndicators = {
  AgentActivityType.thinking: ActivityIndicatorConfig(
    icon: '💭',
    color: '#8B5CF6',
    animation: 'pulse',
    description: '思考中',
  ),
  AgentActivityType.executing: ActivityIndicatorConfig(
    icon: '⚡',
    color: '#F59E0B',
    animation: 'progress',
    description: '执行工具',
  ),
  AgentActivityType.outputting: ActivityIndicatorConfig(
    icon: '💬',
    color: '#10B981',
    animation: 'typewriter',
    description: '输出内容',
  ),
  AgentActivityType.waiting: ActivityIndicatorConfig(
    icon: '⏸️',
    color: '#6B7280',
    animation: 'blink',
    description: '等待权限',
  ),
  AgentActivityType.idle: ActivityIndicatorConfig(
    icon: '😴',
    color: '#9CA3AF',
    animation: 'none',
    description: '空闲',
  ),
  AgentActivityType.error: ActivityIndicatorConfig(
    icon: '❌',
    color: '#EF4444',
    animation: 'shake',
    description: '错误',
  ),
};

/// 活动指示器配置
class ActivityIndicatorConfig {
  final String icon;
  final String color;
  final String animation;
  final String description;

  ActivityIndicatorConfig({
    required this.icon,
    required this.color,
    required this.animation,
    required this.description,
  });
}
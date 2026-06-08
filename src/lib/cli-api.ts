// ACP CLI API - Command-line interface wrapper for ACP operations
// This provides the API that would be exposed via Tauri commands for CLI access

import { invokeOrProxy } from './host';
import { type ACPCapabilities, DEFAULT_ACP_CAPABILITIES } from './acp-protocol/spec';

// ============================================================
// CLI Configuration Commands
// ============================================================

export async function cliGetVersion(): Promise<string> {
  return invokeOrProxy('cli_get_version');
}

export async function cliGetConfig(key?: string): Promise<Record<string, unknown>> {
  return invokeOrProxy('cli_get_config', { key });
}

export async function cliSetConfig(key: string, value: unknown): Promise<void> {
  return invokeOrProxy('cli_set_config', { key, value });
}

// ============================================================
// ACP Connection Commands
// ============================================================

export interface ACPConnectionInfo {
  id: string;
  url: string;
  status: 'connected' | 'disconnected' | 'error';
  capabilities?: ACPCapabilities;
  sessions: string[];
}

export async function cliConnect(url: string): Promise<ACPConnectionInfo> {
  return invokeOrProxy('cli_acp_connect', { url });
}

export async function cliDisconnect(connectionId: string): Promise<void> {
  return invokeOrProxy('cli_acp_disconnect', { connection_id: connectionId });
}

export async function cliListConnections(): Promise<ACPConnectionInfo[]> {
  return invokeOrProxy('cli_acp_list_connections');
}

export async function cliGetConnection(connectionId: string): Promise<ACPConnectionInfo> {
  return invokeOrProxy('cli_acp_get_connection', { connection_id: connectionId });
}

// ============================================================
// Session Commands
// ============================================================

export interface CLISession {
  id: string;
  status: 'active' | 'paused' | 'closed';
  createdAt: number;
}

export async function cliCreateSession(connectionId: string): Promise<string> {
  return invokeOrProxy('cli_acp_create_session', { connection_id: connectionId });
}

export async function cliResumeSession(sessionId: string): Promise<CLISession> {
  return invokeOrProxy('cli_acp_resume_session', { session_id: sessionId });
}

export async function cliCloseSession(sessionId: string): Promise<void> {
  return invokeOrProxy('cli_acp_close_session', { session_id: sessionId });
}

export async function cliListSessions(connectionId?: string): Promise<CLISession[]> {
  return invokeOrProxy('cli_acp_list_sessions', { connection_id: connectionId });
}

// ============================================================
// Task Execution Commands
// ============================================================

export interface CLITask {
  id: string;
  sessionId: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  result?: string;
  progress?: number;
}

export async function cliRunTask(sessionId: string, prompt: string): Promise<CLITask> {
  return invokeOrProxy('cli_run_task', { session_id: sessionId, prompt });
}

export async function cliRunWorkflow(sessionId: string, workflowPath: string): Promise<CLITask[]> {
  return invokeOrProxy('cli_run_workflow', { session_id: sessionId, workflow_path: workflowPath });
}

export async function cliGetTaskStatus(taskId: string): Promise<CLITask> {
  return invokeOrProxy('cli_get_task_status', { task_id: taskId });
}

export async function cliCancelTask(taskId: string): Promise<void> {
  return invokeOrProxy('cli_cancel_task', { task_id: taskId });
}

export async function cliListTasks(sessionId?: string): Promise<CLITask[]> {
  return invokeOrProxy('cli_list_tasks', { session_id: sessionId });
}

// ============================================================
// Agent Commands
// ============================================================

export interface CLIAgent {
  id: string;
  name: string;
  type: 'stdio' | 'websocket' | 'http';
  capabilities: string[];
  status: 'idle' | 'busy' | 'error';
}

export async function cliRegisterAgent(type: string, config: Record<string, unknown>): Promise<CLIAgent> {
  return invokeOrProxy('cli_register_agent', { agent_type: type, config });
}

export async function cliDeregisterAgent(agentId: string): Promise<void> {
  return invokeOrProxy('cli_deregister_agent', { agent_id: agentId });
}

export async function cliListAgents(): Promise<CLIAgent[]> {
  return invokeOrProxy('cli_list_agents');
}

// ============================================================
// Skill Commands
// ============================================================

export interface CLISkill {
  id: string;
  name: string;
  version: string;
  description: string;
  capabilities: string[];
}

export async function cliListSkills(): Promise<CLISkill[]> {
  return invokeOrProxy('cli_list_skills');
}

export async function cliInstallSkill(name: string, source?: string): Promise<CLISkill> {
  return invokeOrProxy('cli_install_skill', { name, source });
}

export async function cliUninstallSkill(skillId: string): Promise<void> {
  return invokeOrProxy('cli_uninstall_skill', { skill_id: skillId });
}

export async function cliEvolveSkill(): Promise<void> {
  return invokeOrProxy('cli_evolve_skill');
}

// ============================================================
// Swarm Commands
// ============================================================

export interface CLISwarmStatus {
  topology: 'star' | 'chain' | 'mesh' | 'tree' | 'ring';
  queenAgent?: string;
  workers: string[];
  activeTasks: number;
}

export async function cliGetSwarmStatus(): Promise<CLISwarmStatus> {
  return invokeOrProxy('cli_get_swarm_status');
}

export async function cliRegisterSwarmAgent(agentType: string, role: 'queen' | 'worker'): Promise<void> {
  return invokeOrProxy('cli_register_swarm_agent', { agent_type: agentType, role });
}

// ============================================================
// MCP Commands
// ============================================================

export async function cliAddMcpServer(url: string, name?: string): Promise<void> {
  return invokeOrProxy('cli_add_mcp_server', { url, name });
}

export async function cliRemoveMcpServer(serverId: string): Promise<void> {
  return invokeOrProxy('cli_remove_mcp_server', { server_id: serverId });
}

export async function cliListMcpServers(): Promise<{ id: string; name: string; url: string; status: string }[]> {
  return invokeOrProxy('cli_list_mcp_servers');
}

// ============================================================
// Hook Commands
// ============================================================

export interface CLIHook {
  id: string;
  type: 'pre-execution' | 'post-execution' | 'on-error' | 'on-schedule' | 'on-thought';
  name: string;
  enabled: boolean;
}

export async function cliListHooks(): Promise<CLIHook[]> {
  return invokeOrProxy('cli_list_hooks');
}

export async function cliEnableHook(hookId: string): Promise<void> {
  return invokeOrProxy('cli_enable_hook', { hook_id: hookId });
}

export async function cliDisableHook(hookId: string): Promise<void> {
  return invokeOrProxy('cli_disable_hook', { hook_id: hookId });
}
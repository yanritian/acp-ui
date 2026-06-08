// src/lib/orchestration-api.ts
// 前端 API 封装：对接 agent_orchestration / approval_engine / sync_engine 后端命令

import { invokeOrProxy } from './host';

// ============================================================
// Agent Orchestration (12 commands from agent_orchestration.rs)
// ============================================================

export interface OrchestratorAgent {
  id: string;
  name: string;
  capabilities: string[];
  cli_command?: string;
  cli_args?: string[];
  status: string;
}

export interface OrchestratorTask {
  id: string;
  agent_id: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  result?: string;
  created_at: string;
  completed_at?: string;
}

export interface WorkflowStage {
  stage_id: string;
  stage_name: string;
  agent_id?: string;
  status: string;
  input_data?: Record<string, unknown>;
  output_data?: Record<string, unknown>;
}

// Agent 注册表
export async function registerAgent(agent: Partial<OrchestratorAgent>): Promise<void> {
  return invokeOrProxy('orchestration_register_agent', { agent });
}

export async function listOrchestratorAgents(): Promise<OrchestratorAgent[]> {
  return invokeOrProxy('orchestration_list_agents');
}

export async function selectBestAgent(taskDescription: string): Promise<OrchestratorAgent> {
  return invokeOrProxy('orchestration_select_agent', { task_description: taskDescription });
}

// 任务执行
export async function executeTask(agentId: string, description: string): Promise<OrchestratorTask> {
  return invokeOrProxy('orchestration_execute_task', { agent_id: agentId, description });
}

export async function executeTaskAuto(description: string): Promise<OrchestratorTask> {
  return invokeOrProxy('orchestration_execute_task_auto', { description });
}

export async function getTaskStatus(taskId: string): Promise<OrchestratorTask> {
  return invokeOrProxy('orchestration_get_task_status', { task_id: taskId });
}

export async function cancelTask(taskId: string): Promise<void> {
  return invokeOrProxy('orchestration_cancel_task', { task_id: taskId });
}

export async function listActiveTasks(): Promise<OrchestratorTask[]> {
  return invokeOrProxy('orchestration_list_active_tasks');
}

export async function listTaskHistory(page?: number, pageSize?: number): Promise<OrchestratorTask[]> {
  return invokeOrProxy('orchestration_list_task_history', { page, page_size: pageSize });
}

// 工作流
export async function executeWorkflowStage(workflowId: string, stage: WorkflowStage): Promise<WorkflowStage> {
  return invokeOrProxy('orchestration_execute_workflow_stage', { workflow_id: workflowId, stage });
}

export async function executeFullWorkflow(workflowId: string, stages: WorkflowStage[]): Promise<OrchestratorTask[]> {
  return invokeOrProxy('orchestration_execute_full_workflow', { workflow_id: workflowId, stages });
}

// 种子数据
export async function seedDefaultAgents(): Promise<void> {
  return invokeOrProxy('orchestration_seed_default_agents');
}

// ============================================================
// Approval Engine (4 commands from approval_engine.rs)
// ============================================================

export interface ApprovalRequest {
  id: string;
  task_id: string;
  stage_name: string;
  request_type: string;
  description: string;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  created_at: string;
  decided_at?: string;
  decision?: string;
  feedback?: string;
}

export interface ApprovalStats {
  total: number;
  pending: number;
  approved: number;
  rejected: number;
  expired: number;
}

export async function getPendingApprovals(): Promise<ApprovalRequest[]> {
  return invokeOrProxy('approval_get_pending');
}

export async function getApprovalRequest(requestId: string): Promise<ApprovalRequest> {
  return invokeOrProxy('approval_get_request', { request_id: requestId });
}

export async function decideApproval(
  requestId: string,
  decision: 'approved' | 'rejected',
  feedback?: string,
): Promise<void> {
  return invokeOrProxy('approval_decide', { request_id: requestId, decision, feedback });
}

export async function getApprovalStats(): Promise<ApprovalStats> {
  return invokeOrProxy('approval_get_stats');
}

// ============================================================
// Sync Engine (7 commands from sync_engine.rs)
// ============================================================

export interface SyncEntity {
  entity_type: string;
  entity_id: string;
  data: Record<string, unknown>;
  version: number;
  source?: string;
  updated_at?: string;
}

export async function syncPutEntity(entity: SyncEntity): Promise<void> {
  return invokeOrProxy('sync_put_entity', { entity });
}

export async function syncGetEntity(entityType: string, entityId: string): Promise<SyncEntity | null> {
  return invokeOrProxy('sync_get_entity', { entity_type: entityType, entity_id: entityId });
}

export async function syncDeleteEntity(entityType: string, entityId: string): Promise<void> {
  return invokeOrProxy('sync_delete_entity', { entity_type: entityType, entity_id: entityId });
}

export async function syncListEntities(entityType: string): Promise<SyncEntity[]> {
  return invokeOrProxy('sync_list_entities', { entity_type: entityType });
}

export async function syncGetStats(): Promise<Record<string, number>> {
  return invokeOrProxy('sync_get_stats');
}

export async function syncReceiveEvent(event: Record<string, unknown>): Promise<void> {
  return invokeOrProxy('sync_receive_event', { event });
}

export async function syncFlushOffline(): Promise<void> {
  return invokeOrProxy('sync_flush_offline');
}
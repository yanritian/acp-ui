// Workflow Definition DSL - YAML/JSON declarative workflow definitions
// Parses, validates, and loads workflows from JSON format.

export type WorkflowFailurePolicy = 'abort' | 'retry' | 'fallback_to_manual' | 'continue';

export interface StageDefinition {
  id: string;
  name: string;
  agentId?: string;
  capability?: string;
  inputs: string[];
  outputs: string[];
  timeout?: number;
  requiresApproval?: boolean;
  retryCount?: number;
  condition?: string;
}

export interface WorkflowDefinition {
  id: string;
  name: string;
  description: string;
  version: string;
  stages: StageDefinition[];
  onFailure: WorkflowFailurePolicy;
}

// ---------- Result type ----------

export type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };

// ---------- Validation helpers ----------

const REQUIRED_STAGE_FIELDS: (keyof StageDefinition)[] = ['id', 'name', 'inputs', 'outputs'];
const VALID_FAILURE_POLICIES: WorkflowFailurePolicy[] = ['abort', 'retry', 'fallback_to_manual', 'continue'];

function nonBlank(value: string, label: string): string | null {
  if (!value || typeof value !== 'string' || value.trim().length === 0) {
    return `${label} is required and must be non-empty`;
  }
  return null;
}

function validateStage(stage: unknown, index: number): string[] {
  const errors: string[] = [];

  if (!stage || typeof stage !== 'object') {
    errors.push(`stages[${index}] must be an object`);
    return errors;
  }

  const s = stage as Record<string, unknown>;

  for (const field of REQUIRED_STAGE_FIELDS) {
    const err = nonBlank(s[field] as string, `stages[${index}].${field}`);
    if (err) errors.push(err);
  }

  if (typeof s.inputs !== 'object' || !Array.isArray(s.inputs)) {
    errors.push(`stages[${index}].inputs must be an array`);
  }
  if (typeof s.outputs !== 'object' || !Array.isArray(s.outputs)) {
    errors.push(`stages[${index}].outputs must be an array`);
  }

  if (s.timeout !== undefined) {
    if (typeof s.timeout !== 'number' || s.timeout <= 0) {
      errors.push(`stages[${index}].timeout must be a positive number`);
    }
  }

  if (s.retryCount !== undefined) {
    if (typeof s.retryCount !== 'number' || s.retryCount < 0 || !Number.isInteger(s.retryCount)) {
      errors.push(`stages[${index}].retryCount must be a non-negative integer`);
    }
  }

  if (s.requiresApproval !== undefined && typeof s.requiresApproval !== 'boolean') {
    errors.push(`stages[${index}].requiresApproval must be a boolean`);
  }

  if (s.condition !== undefined && typeof s.condition !== 'string') {
    errors.push(`stages[${index}].condition must be a string expression`);
  }

  return errors;
}

export function validateWorkflow(workflow: WorkflowDefinition): string[] {
  const errors: string[] = [];

  const idErr = nonBlank(workflow.id, 'id');
  if (idErr) errors.push(idErr);

  const nameErr = nonBlank(workflow.name, 'name');
  if (nameErr) errors.push(nameErr);

  const descErr = nonBlank(workflow.description, 'description');
  if (descErr) errors.push(descErr);

  const versionErr = nonBlank(workflow.version, 'version');
  if (versionErr) errors.push(versionErr);

  if (!VALID_FAILURE_POLICIES.includes(workflow.onFailure)) {
    errors.push(
      `onFailure must be one of: ${VALID_FAILURE_POLICIES.join(', ')}, got "${workflow.onFailure}"`
    );
  }

  if (!Array.isArray(workflow.stages) || workflow.stages.length === 0) {
    errors.push('stages must be a non-empty array');
  } else {
    // Check for duplicate stage IDs
    const stageIds = new Set<string>();
    for (let i = 0; i < workflow.stages.length; i++) {
      const stage = workflow.stages[i];
      if (stageIds.has(stage.id)) {
        errors.push(`duplicate stage id: "${stage.id}"`);
      }
      stageIds.add(stage.id);
      errors.push(...validateStage(stage, i));
    }
  }

  return errors;
}

export function parseWorkflow(json: string): Result<WorkflowDefinition, string> {
  let parsed: unknown;

  try {
    parsed = JSON.parse(json);
  } catch (e) {
    return { ok: false, error: `Invalid JSON: ${e instanceof Error ? e.message : String(e)}` };
  }

  if (!parsed || typeof parsed !== 'object') {
    return { ok: false, error: 'Workflow must be a JSON object' };
  }

  const wf = parsed as WorkflowDefinition;

  // Apply defaults for optional fields
  const definition: WorkflowDefinition = {
    id: wf.id || '',
    name: wf.name || '',
    description: wf.description || '',
    version: wf.version || '1.0.0',
    onFailure: wf.onFailure || 'abort',
    stages: Array.isArray(wf.stages)
      ? wf.stages.map((s) => ({
          id: s.id,
          name: s.name,
          agentId: s.agentId,
          capability: s.capability,
          inputs: Array.isArray(s.inputs) ? s.inputs : [],
          outputs: Array.isArray(s.outputs) ? s.outputs : [],
          timeout: s.timeout,
          requiresApproval: s.requiresApproval ?? false,
          retryCount: s.retryCount ?? 0,
          condition: s.condition,
        }))
      : [],
  };

  const errors = validateWorkflow(definition);
  if (errors.length > 0) {
    return { ok: false, error: errors.join('; ') };
  }

  return { ok: true, value: definition };
}

// ---------- Default workflow ----------

const DEFAULT_STAGES: StageDefinition[] = [
  { id: 'plan', name: 'Plan', inputs: ['task'], outputs: ['plan'], requiresApproval: true },
  { id: 'design', name: 'Design', inputs: ['plan'], outputs: ['design'] },
  { id: 'implement', name: 'Implement', inputs: ['design'], outputs: ['code'] },
  { id: 'test', name: 'Test', inputs: ['code'], outputs: ['test_results'] },
  { id: 'review', name: 'Review', inputs: ['code', 'test_results'], outputs: ['review_feedback'], requiresApproval: true },
  { id: 'refine', name: 'Refine', inputs: ['review_feedback'], outputs: ['refined_code'], condition: "review_feedback.needs_changes === true" },
  { id: 'deploy', name: 'Deploy', inputs: ['refined_code'], outputs: ['deploy_status'] },
  { id: 'verify', name: 'Verify', inputs: ['deploy_status'], outputs: ['verification_report'] },
];

export function createDefaultWorkflow(name: string): WorkflowDefinition {
  const id = `wf-${Date.now()}-${crypto.randomUUID().slice(0, 8)}`;
  return {
    id,
    name,
    description: `Default 8-stage workflow: ${name}`,
    version: '1.0.0',
    stages: DEFAULT_STAGES.map((s, i) => ({ ...s, id: `${s.id}-${i}` })),
    onFailure: 'abort',
  };
}

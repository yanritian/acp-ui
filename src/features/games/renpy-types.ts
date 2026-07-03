// Ren'Py TypeScript Type Definitions
// Auto-generated for Tauri commands

// Request/Response types

export interface RenPyVersionRequest {}

export interface RenPyVersionResponse {
  success: boolean;
  version: string | null;
  error: string | null;
}

export interface RenPyCreateProjectRequest {
  name: string;
  path: string;
}

export interface RenPyCreateProjectResponse {
  success: boolean;
  projectPath: string | null;
  error: string | null;
}

export interface RenPyGenerateScriptRequest {
  projectPath: string;
  storySpec: StorySpec;
}

export interface RenPyGenerateScriptResponse {
  success: boolean;
  scriptPath: string | null;
  error: string | null;
}

export interface RenPyRunGameRequest {
  projectPath: string;
}

export interface RenPyRunGameResponse {
  success: boolean;
  processId: number | null;
  error: string | null;
}

export interface RenPyCompileGameRequest {
  projectPath: string;
}

export interface RenPyCompileGameResponse {
  success: boolean;
  output: string | null;
  error: string | null;
}

export interface RenPyLintGameRequest {
  projectPath: string;
}

export interface RenPyLintGameResponse {
  success: boolean;
  output: string | null;
  error: string | null;
}

export interface RenPyDetectProjectRequest {
  path: string;
}

export interface RenPyDetectProjectResponse {
  success: boolean;
  isProject: boolean;
  projectName: string | null;
  error: string | null;
}

// Story specification types

export interface StorySpec {
  title: string;
  characters: Character[];
  scenes: Scene[];
  backgrounds: Background[];
}

export interface Character {
  id: string;
  name: string;
  color: string;
  expressions: Expression[];
}

export interface Expression {
  name: string;
  imageFilename: string;
}

export interface Scene {
  label: string;
  background: string | null;
  dialogues: Dialogue[];
  menu: Menu | null;
  nextLabel: string | null;
}

export interface Dialogue {
  speaker: string | null;
  text: string;
  characterShows: CharacterShow[];
}

export interface CharacterShow {
  characterId: string;
  expression: string;
  position: string;
}

export interface Menu {
  prompt: string | null;
  choices: Choice[];
}

export interface Choice {
  text: string;
  targetLabel: string;
}

export interface Background {
  id: string;
  imageFilename: string;
}

// Platform types

export type RenPyPlatform = 'win' | 'mac' | 'linux' | 'android' | 'ios' | 'web';

// Helper types

export interface RenPyProject {
  path: string;
  name: string;
  version: string;
  isProject: boolean;
}

export interface RenPyGameStatus {
  processId: number;
  isRunning: boolean;
  startedAt: number;
}

// Error types

export class RenPyError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'RenPyError';
  }
}

// Type guards

export function isRenPyVersionResponse(obj: unknown): obj is RenPyVersionResponse {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'success' in obj &&
    typeof (obj as RenPyVersionResponse).success === 'boolean'
  );
}

export function isStorySpec(obj: unknown): obj is StorySpec {
  return (
    typeof obj === 'object' &&
    obj !== null &&
    'title' in obj &&
    'characters' in obj &&
    'scenes' in obj &&
    'backgrounds' in obj &&
    Array.isArray((obj as StorySpec).characters) &&
    Array.isArray((obj as StorySpec).scenes) &&
    Array.isArray((obj as StorySpec).backgrounds)
  );
}

// Default values

export const DEFAULT_CHARACTER: Character = {
  id: 'char_1',
  name: 'Character 1',
  color: '#c8ffc8',
  expressions: [
    { name: 'happy', imageFilename: 'char_1_happy.png' },
    { name: 'sad', imageFilename: 'char_1_sad.png' },
  ],
};

export const DEFAULT_SCENE: Scene = {
  label: 'start',
  background: null,
  dialogues: [],
  menu: null,
  nextLabel: null,
};

export const DEFAULT_BACKGROUND: Background = {
  id: 'bg_1',
  imageFilename: 'bg_1.png',
};

// Validation functions

export function validateStorySpec(spec: StorySpec): string[] {
  const errors: string[] = [];

  // Check title
  if (!spec.title || spec.title.trim() === '') {
    errors.push('Story must have a title');
  }

  // Check characters
  if (spec.characters.length === 0) {
    errors.push('Story must have at least one character');
  }

  // Check scenes
  if (spec.scenes.length === 0) {
    errors.push('Story must have at least one scene');
  }

  // Check start label
  const hasStartLabel = spec.scenes.some(s => s.label === 'start');
  if (!hasStartLabel) {
    errors.push('Story must have a scene with label "start"');
  }

  // Check for duplicate labels
  const labels = spec.scenes.map(s => s.label);
  const duplicateLabels = labels.filter((label, index) => labels.indexOf(label) !== index);
  if (duplicateLabels.length > 0) {
    errors.push(`Duplicate scene labels found: ${duplicateLabels.join(', ')}`);
  }

  // Check jump targets
  const labelSet = new Set(labels);
  for (const scene of spec.scenes) {
    if (scene.nextLabel && !labelSet.has(scene.nextLabel)) {
      errors.push(`Scene "${scene.label}" jumps to non-existent label: ${scene.nextLabel}`);
    }

    if (scene.menu) {
      for (const choice of scene.menu.choices) {
        if (!labelSet.has(choice.targetLabel)) {
          errors.push(`Choice "${choice.text}" in scene "${scene.label}" jumps to non-existent label: ${choice.targetLabel}`);
        }
      }
    }
  }

  // Check character references
  const characterIds = new Set(spec.characters.map(c => c.id));
  for (const scene of spec.scenes) {
    for (const dialogue of scene.dialogues) {
      if (dialogue.speaker && !characterIds.has(dialogue.speaker)) {
        errors.push(`Dialogue in scene "${scene.label}" references non-existent character: ${dialogue.speaker}`);
      }

      for (const charShow of dialogue.characterShows) {
        if (!characterIds.has(charShow.characterId)) {
          errors.push(`Character show in scene "${scene.label}" references non-existent character: ${charShow.characterId}`);
        }
      }
    }
  }

  // Check background references
  const backgroundIds = new Set(spec.backgrounds.map(b => b.id));
  for (const scene of spec.scenes) {
    if (scene.background && !backgroundIds.has(scene.background)) {
      errors.push(`Scene "${scene.label}" uses non-existent background: ${scene.background}`);
    }
  }

  return errors;
}

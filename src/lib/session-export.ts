/**
 * Session Export Utilities
 *
 * Provides functions for exporting sessions to different formats.
 */

import type { SavedSession, ChatMessage } from '@/lib/types';

/**
 * Export session to Markdown format
 */
export function exportToMarkdown(
  session: SavedSession,
  messages: ChatMessage[]
): string {
  const lines: string[] = [];

  // Header
  lines.push(`# ${session.title}`);
  lines.push('');
  lines.push(`**Agent:** ${session.agentName}`);
  lines.push(`**Date:** ${new Date(session.lastUpdated).toLocaleString()}`);
  lines.push(`**Working Directory:** ${session.cwd}`);
  lines.push('');

  // Messages
  for (const message of messages) {
    const roleLabel = message.role === 'user' ? 'User' : 'Assistant';
    const timestamp = new Date(message.timestamp).toLocaleString();

    lines.push(`---`);
    lines.push('');
    lines.push(`### ${roleLabel} (${timestamp})`);
    lines.push('');

    // Thought section (if present)
    if (message.thought) {
      lines.push(`> **Thinking:**`);
      lines.push(`> ${message.thought.split('\n').join('\n> ')}`);
      lines.push('');
    }

    // Tool calls (if present)
    if (message.toolCalls?.length) {
      lines.push(`**Tool Calls:**`);
      for (const tc of message.toolCalls) {
        const statusIcon = tc.status === 'completed' ? '✓' : tc.status === 'failed' ? '✗' : '⏳';
        lines.push(`- ${statusIcon} **${tc.title}** (${tc.kind})`);
        if (tc.locations?.length) {
          lines.push(`  - File: ${tc.locations[0].path}`);
        }
      }
      lines.push('');
    }

    // Content
    if (message.content) {
      lines.push(message.content);
      lines.push('');
    }
  }

  return lines.join('\n');
}

/**
 * Export session to JSON format
 */
export function exportToJson(
  session: SavedSession,
  messages: ChatMessage[]
): string {
  const exportData = {
    session: {
      id: session.id,
      title: session.title,
      agentName: session.agentName,
      lastUpdated: session.lastUpdated,
      cwd: session.cwd,
      pinned: session.pinned ?? false,
    },
    messages: messages.map(msg => ({
      id: msg.id,
      role: msg.role,
      content: msg.content,
      thought: msg.thought,
      timestamp: msg.timestamp,
      toolCalls: msg.toolCalls,
    })),
    exportedAt: Date.now(),
    exportedBy: 'ACP-UI',
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Copy session content to clipboard
 */
export async function copyToClipboard(content: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(content);
    return true;
  } catch (e) {
    console.error('Failed to copy to clipboard:', e);
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = content;
    textarea.style.position = 'fixed';
    textarea.style.left = '-9999px';
    document.body.appendChild(textarea);
    textarea.select();
    try {
      document.execCommand('copy');
      return true;
    } catch (fallbackError) {
      console.error('Fallback copy failed:', fallbackError);
      return false;
    } finally {
      document.body.removeChild(textarea);
    }
  }
}

/**
 * Download content as a file
 */
export function downloadFile(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Generate a safe filename from session title
 */
export function generateFilename(session: SavedSession, extension: string): string {
  const safeTitle = session.title
    .replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_') // Keep alphanumeric and Chinese characters
    .replace(/_+/g, '_')
    .slice(0, 50);
  const dateStr = new Date(session.lastUpdated).toISOString().slice(0, 10);
  return `${safeTitle}_${dateStr}.${extension}`;
}

/**
 * Time group for session display
 */
export type TimeGroup = 'pinned' | 'today' | 'yesterday' | 'thisWeek' | 'older';

/**
 * Get time group for a session based on lastUpdated timestamp
 */
export function getTimeGroup(session: SavedSession): TimeGroup {
  if (session.pinned) {
    return 'pinned';
  }

  const now = Date.now();
  const lastUpdated = session.lastUpdated;
  const dayMs = 24 * 60 * 60 * 1000;

  const todayStart = new Date().setHours(0, 0, 0, 0);
  const yesterdayStart = todayStart - dayMs;
  const weekStart = todayStart - 7 * dayMs;

  if (lastUpdated >= todayStart) {
    return 'today';
  } else if (lastUpdated >= yesterdayStart) {
    return 'yesterday';
  } else if (lastUpdated >= weekStart) {
    return 'thisWeek';
  } else {
    return 'older';
  }
}

/**
 * Get display label for time group
 */
export function getTimeGroupLabel(group: TimeGroup, t: (key: string) => string): string {
  switch (group) {
    case 'pinned':
      return t('sessionList.groupPinned');
    case 'today':
      return t('sessionList.groupToday');
    case 'yesterday':
      return t('sessionList.groupYesterday');
    case 'thisWeek':
      return t('sessionList.groupThisWeek');
    case 'older':
      return t('sessionList.groupOlder');
    default:
      return '';
  }
}

/**
 * Group sessions by time
 */
export function groupSessionsByTime(
  sessions: SavedSession[],
  t: (key: string) => string
): Map<TimeGroup, SavedSession[]> {
  const groups = new Map<TimeGroup, SavedSession[]>();

  // Initialize all groups
  const allGroups: TimeGroup[] = ['pinned', 'today', 'yesterday', 'thisWeek', 'older'];
  for (const group of allGroups) {
    groups.set(group, []);
  }

  // Assign sessions to groups
  for (const session of sessions) {
    const group = getTimeGroup(session);
    groups.get(group)?.push(session);
  }

  // Sort sessions within each group by lastUpdated (descending)
  for (const [, sessions] of groups) {
    sessions.sort((a, b) => b.lastUpdated - a.lastUpdated);
  }

  return groups;
}
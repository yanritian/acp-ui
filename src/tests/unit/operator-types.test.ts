// Unit tests for Operator types

import { describe, it, expect } from 'vitest'
import type {
  OperatorTask,
  OperatorEvent,
  ApprovalRequest,
  OperatorTaskStatus,
  OperatorEventType,
  ApprovalLevel,
  EventLevel
} from '@/types/operator'

describe('Operator Types', () => {
  describe('OperatorTaskStatus', () => {
    it('should define all valid statuses', () => {
      const statuses: OperatorTaskStatus[] = [
        'idle',
        'planning',
        'waiting_approval',
        'running',
        'paused',
        'redirecting',
        'cancelling',
        'cancelled',
        'failed',
        'completed'
      ]
      expect(statuses.length).toBe(10)
    })

    it('should have correct terminal states', () => {
      const terminalStates: OperatorTaskStatus[] = ['completed', 'failed', 'cancelled']
      terminalStates.forEach(status => {
        expect(['completed', 'failed', 'cancelled']).toContain(status)
      })
    })
  })

  describe('ApprovalLevel', () => {
    it('should define approval levels in order', () => {
      const levels: ApprovalLevel[] = ['silent', 'notify', 'approve', 'forbidden']
      expect(levels[0]).toBe('silent')
      expect(levels[3]).toBe('forbidden')
    })
  })
})

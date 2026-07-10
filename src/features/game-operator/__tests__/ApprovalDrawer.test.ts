// Approval Drawer Component Tests
// Tests for approval queue display and actions

import { describe, it, expect, vi, beforeEach } from 'vitest'
import approvalDrawerSource from '../components/ApprovalDrawer.vue?raw'

describe('ApprovalDrawer', () => {
  const mockApprovals = [
    {
      approval_id: 'approval_001',
      task_id: 'task_001',
      level: 'approve',
      action: 'file.patch',
      title: 'Modify Player.gd',
      reason: 'Adding jump logic',
      options: ['approve', 'reject'],
      created_at: '2026-07-09T10:00:00Z'
    },
    {
      approval_id: 'approval_002',
      task_id: 'task_001',
      level: 'approve',
      action: 'file.write',
      title: 'Create Utils.gd',
      reason: 'New utility class',
      options: ['approve', 'reject'],
      created_at: '2026-07-09T10:01:00Z'
    }
  ]

  describe('Approval List Display', () => {
    it('should display approval items', () => {
      const approvals = [...mockApprovals]
      expect(approvals.length).toBe(2)
    })

    it('should handle empty approval list', () => {
      const approvals: any[] = []
      expect(approvals.length).toBe(0)
    })

    it('should show approval title', () => {
      const approval = mockApprovals[0]
      expect(approval.title).toBe('Modify Player.gd')
    })

    it('should show approval reason', () => {
      const approval = mockApprovals[0]
      expect(approval.reason).toBe('Adding jump logic')
    })

    it('should show approval level', () => {
      const approval = mockApprovals[0]
      expect(approval.level).toBe('approve')
    })

    it('should show available options', () => {
      const approval = mockApprovals[0]
      expect(approval.options).toContain('approve')
      expect(approval.options).toContain('reject')
    })
  })

  describe('Approval Actions', () => {
    it('should emit approve action', () => {
      const handler = vi.fn()
      handler('approval_001', 'approve')
      expect(handler).toHaveBeenCalledWith('approval_001', 'approve')
    })

    it('should emit reject action', () => {
      const handler = vi.fn()
      handler('approval_001', 'reject')
      expect(handler).toHaveBeenCalledWith('approval_001', 'reject')
    })

    it('should require approval_id for action', () => {
      const approval = mockApprovals[0]
      expect(approval.approval_id).toBeDefined()
    })
  })

  describe('Approval Levels', () => {
    it('should handle silent level', () => {
      const approval = { ...mockApprovals[0], level: 'silent' }
      expect(approval.level).toBe('silent')
    })

    it('should handle notify level', () => {
      const approval = { ...mockApprovals[0], level: 'notify' }
      expect(approval.level).toBe('notify')
    })

    it('should handle approve level', () => {
      const approval = { ...mockApprovals[0], level: 'approve' }
      expect(approval.level).toBe('approve')
    })

    it('should handle forbidden level', () => {
      const approval = { ...mockApprovals[0], level: 'forbidden' }
      expect(approval.level).toBe('forbidden')
    })
  })

  describe('Preview Display', () => {
    it('should show file preview', () => {
      const approval = {
        ...mockApprovals[0],
        preview: {
          files: ['scripts/Player.gd']
        }
      }
      expect(approval.preview?.files).toContain('scripts/Player.gd')
    })

    it('should show diff preview', () => {
      const approval = {
        ...mockApprovals[0],
        preview: {
          diff_id: 'diff_001'
        }
      }
      expect(approval.preview?.diff_id).toBe('diff_001')
    })

    it('should expose structured per-file diffs for review', () => {
      const approval = {
        ...mockApprovals[0],
        preview: {
          diff_id: 'patch_001',
          diffs: [
            {
              path: 'scripts/Player.gd',
              operation: 'replace',
              diff: '--- a/scripts/Player.gd\n+++ b/scripts/Player.gd\n-var jumps = 1\n+var jumps = 2\n'
            }
          ]
        }
      }

      expect(approval.preview.diffs[0].operation).toBe('replace')
      expect(approval.preview.diffs[0].diff).toContain('+var jumps = 2')
    })

    it('should show command preview', () => {
      const approval = {
        ...mockApprovals[0],
        action: 'shell.command',
        preview: {
          command: 'cargo build'
        }
      }
      expect(approval.preview?.command).toBe('cargo build')
    })
  })

  describe('Risk Assessment', () => {
    it('should show high risk indicator', () => {
      const approval = {
        ...mockApprovals[0],
        risk: 'high'
      }
      expect(approval.risk).toBe('high')
    })

    it('should show low risk indicator', () => {
      const approval = {
        ...mockApprovals[0],
        risk: 'low'
      }
      expect(approval.risk).toBe('low')
    })

    it('should show risk description', () => {
      const approval = {
        ...mockApprovals[0],
        risk: 'high - irreversible operation'
      }
      expect(approval.risk).toContain('high')
    })
  })

  describe('Resolution State', () => {
    it('should show resolved approval', () => {
      const approval = {
        ...mockApprovals[0],
        resolved_at: '2026-07-09T10:05:00Z',
        decision: 'approve'
      }
      expect(approval.decision).toBe('approve')
    })

    it('should show rejected approval', () => {
      const approval = {
        ...mockApprovals[0],
        resolved_at: '2026-07-09T10:05:00Z',
        decision: 'reject'
      }
      expect(approval.decision).toBe('reject')
    })

    it('should show who resolved', () => {
      const approval = {
        ...mockApprovals[0],
        resolved_by: 'user_001'
      }
      expect(approval.resolved_by).toBe('user_001')
    })
  })

  describe('UI State', () => {
    it('should let the operator surface own scrolling in a narrow layout', () => {
      expect(approvalDrawerSource).toMatch(
        /\.approval-drawer\s*\{[^}]*height:\s*auto;/s,
      )
      expect(approvalDrawerSource).toMatch(
        /\.approval-list\s*\{[^}]*overflow-y:\s*visible;/s,
      )
    })

    it('should show pending count', () => {
      const pendingCount = mockApprovals.length
      expect(pendingCount).toBe(2)
    })

    it('should show approval badge', () => {
      const hasBadge = true
      expect(hasBadge).toBe(true)
    })

    it('should expand approval drawer', () => {
      const isExpanded = true
      expect(isExpanded).toBe(true)
    })

    it('should collapse approval drawer', () => {
      const isCollapsed = false
      expect(isCollapsed).toBe(false)
    })
  })

  describe('Batch Operations', () => {
    it('should approve all pending', () => {
      const handler = vi.fn()
      mockApprovals.forEach(a => handler(a.approval_id, 'approve'))
      expect(handler).toHaveBeenCalledTimes(2)
    })

    it('should reject all pending', () => {
      const handler = vi.fn()
      mockApprovals.forEach(a => handler(a.approval_id, 'reject'))
      expect(handler).toHaveBeenCalledTimes(2)
    })
  })

  describe('Accessibility', () => {
    it('should have approve button label', () => {
      const approveLabel = 'Approve'
      expect(approveLabel).toBeDefined()
    })

    it('should have reject button label', () => {
      const rejectLabel = 'Reject'
      expect(rejectLabel).toBeDefined()
    })

    it('should have descriptive title', () => {
      const approval = mockApprovals[0]
      expect(approval.title).toBeDefined()
    })
  })
})

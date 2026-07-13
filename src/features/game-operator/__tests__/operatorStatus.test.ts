import { describe, expect, it } from 'vitest'
import { operatorStatusKey } from '../operatorStatus'
import approvalDrawerSource from '../components/ApprovalDrawer.vue?raw'
import progressTimelineSource from '../components/ProgressTimeline.vue?raw'

describe('operatorStatusKey', () => {
  it('maps the wire-format waiting_approval status to its locale key', () => {
    expect(operatorStatusKey('waiting_approval')).toBe('waitingApproval')
  })

  it('keeps already normalized status keys unchanged', () => {
    expect(operatorStatusKey('running')).toBe('running')
  })

  it('uses dedicated localized headings for approval and progress panels', () => {
    expect(approvalDrawerSource).toContain("t('gameOperator.approvalQueue')")
    expect(progressTimelineSource).toContain("t('gameOperator.progressTimeline')")
    expect(approvalDrawerSource).not.toContain("t('gameOperator.title')")
    expect(progressTimelineSource).not.toContain("t('gameOperator.title')")
  })

  it('uses a dedicated project path placeholder instead of the task goal example', async () => {
    const gameOperatorViewSource = await import('../views/GameOperatorView.vue?raw')
    expect(gameOperatorViewSource.default).toContain("t('gameOperator.projectPathPlaceholder')")
  })
})

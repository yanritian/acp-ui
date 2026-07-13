/** Maps backend wire-format task states to locale key names. */
export function operatorStatusKey(status: string): string {
  return status === 'waiting_approval' ? 'waitingApproval' : status
}

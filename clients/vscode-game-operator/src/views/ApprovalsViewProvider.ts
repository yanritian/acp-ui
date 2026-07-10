// Approvals View Provider
// Displays pending approvals

import * as vscode from 'vscode'
import { GameOperatorClient, ApprovalRequest } from '../client'

export class ApprovalsViewProvider implements vscode.TreeDataProvider<ApprovalItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<ApprovalItem | undefined | void> = new vscode.EventEmitter<ApprovalItem | undefined | void>()
  readonly onDidChangeTreeData: vscode.Event<ApprovalItem | undefined | void> = this._onDidChangeTreeData.event

  constructor(private client: GameOperatorClient) {}

  refresh(): void {
    this._onDidChangeTreeData.fire()
  }

  getTreeItem(element: ApprovalItem): vscode.TreeItem {
    return element
  }

  async getChildren(element?: ApprovalItem): Promise<ApprovalItem[]> {
    if (!this.client.isConnected()) {
      return []
    }

    if (element) {
      return []
    }

    try {
      const approvals = await this.client.getPendingApprovals()
      return approvals.map(approval => new ApprovalItem(approval))
    } catch (error: any) {
      return []
    }
  }
}

export class ApprovalItem extends vscode.TreeItem {
  constructor(public readonly approval: ApprovalRequest) {
    super(approval.title, vscode.TreeItemCollapsibleState.None)

    this.tooltip = `${approval.level}: ${approval.title}\n${approval.reason}`
    this.description = approval.action

    this.contextValue = 'approval'
    this.iconPath = new vscode.ThemeIcon('bell', new vscode.ThemeColor('charts.orange'))

    // Add command to show diff
    this.command = {
      command: 'gameOperator.showApproval',
      title: 'Show Approval',
      arguments: [approval]
    }
  }
}
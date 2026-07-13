// Task Tree Provider
// Displays tasks in the VSCode tree view

import * as vscode from 'vscode'
import { GameOperatorClient } from '../client'

export class TaskTreeProvider implements vscode.TreeDataProvider<TaskItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<TaskItem | undefined | void> = new vscode.EventEmitter<TaskItem | undefined | void>()
  readonly onDidChangeTreeData: vscode.Event<TaskItem | undefined | void> = this._onDidChangeTreeData.event

  constructor(private client: GameOperatorClient) {}

  refresh(): void {
    this._onDidChangeTreeData.fire()
  }

  getTreeItem(element: TaskItem): vscode.TreeItem {
    return element
  }

  async getChildren(element?: TaskItem): Promise<TaskItem[]> {
    if (!this.client.isConnected()) {
      return [new TaskItem('Not connected', 'connect', vscode.TreeItemCollapsibleState.None)]
    }

    if (element) {
      return []
    }

    try {
      const tasks = await this.client.listTasks()
      return tasks.map(task => new TaskItem(
        task.goal,
        task.task_id,
        vscode.TreeItemCollapsibleState.None,
        task
      ))
    } catch (error: any) {
      return [new TaskItem(`Error: ${error.message}`, 'error', vscode.TreeItemCollapsibleState.None)]
    }
  }
}

export class TaskItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly taskId: string,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    public readonly task?: any,
    public readonly command?: vscode.Command
  ) {
    super(label, collapsibleState)

    this.tooltip = `${this.label}`
    this.description = taskId

    if (task) {
      this.contextValue = 'task'
      this.iconPath = this.getStatusIcon(task.status)
    }
  }

  private getStatusIcon(status: string): vscode.ThemeIcon {
    switch (status) {
      case 'running':
        return new vscode.ThemeIcon('play', new vscode.ThemeColor('charts.green'))
      case 'paused':
        return new vscode.ThemeIcon('debug-pause', new vscode.ThemeColor('charts.yellow'))
      case 'completed':
        return new vscode.ThemeIcon('check', new vscode.ThemeColor('charts.green'))
      case 'failed':
        return new vscode.ThemeIcon('error', new vscode.ThemeColor('charts.red'))
      case 'waiting_approval':
        return new vscode.ThemeIcon('bell', new vscode.ThemeColor('charts.orange'))
      default:
        return new vscode.ThemeIcon('circle-outline')
    }
  }
}

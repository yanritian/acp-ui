// Timeline View Provider
// Displays task events timeline

import * as vscode from 'vscode'
import { GameOperatorClient, OperatorEvent } from '../client'

export class TimelineViewProvider implements vscode.TreeDataProvider<EventItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<EventItem | undefined | void> = new vscode.EventEmitter<EventItem | undefined | void>()
  readonly onDidChangeTreeData: vscode.Event<EventItem | undefined | void> = this._onDidChangeTreeData.event

  constructor(private client: GameOperatorClient) {}

  refresh(): void {
    this._onDidChangeTreeData.fire()
  }

  getTreeItem(element: EventItem): vscode.TreeItem {
    return element
  }

  async getChildren(_element?: EventItem): Promise<EventItem[]> {
    if (!this.client.isConnected()) {
      return []
    }

    // For now, show events for the first task
    try {
      const tasks = await this.client.listTasks()
      if (tasks.length === 0) return []

      const events = await this.client.listEvents(tasks[0].task_id)
      return events.map(event => new EventItem(event))
    } catch (error: any) {
      return []
    }
  }
}

export class EventItem extends vscode.TreeItem {
  constructor(public readonly event: OperatorEvent) {
    super(event.title, vscode.TreeItemCollapsibleState.None)

    this.tooltip = `${event.type}: ${event.title}`
    this.description = new Date(event.timestamp).toLocaleTimeString()

    this.iconPath = this.getLevelIcon(event.level)
  }

  private getLevelIcon(level: string): vscode.ThemeIcon {
    switch (level) {
      case 'error':
        return new vscode.ThemeIcon('error', new vscode.ThemeColor('charts.red'))
      case 'warning':
        return new vscode.ThemeIcon('warning', new vscode.ThemeColor('charts.yellow'))
      case 'info':
        return new vscode.ThemeIcon('info', new vscode.ThemeColor('charts.blue'))
      default:
        return new vscode.ThemeIcon('circle-outline')
    }
  }
}

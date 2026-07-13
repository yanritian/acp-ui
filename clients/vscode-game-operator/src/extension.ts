// Hermes Game Operator VSCode Extension
// Main entry point for the extension

import * as vscode from 'vscode'
import { GameOperatorClient } from './client'
import { TaskTreeProvider } from './views/TaskTreeProvider'
import { TimelineViewProvider } from './views/TimelineViewProvider'
import { ApprovalsViewProvider } from './views/ApprovalsViewProvider'

let client: GameOperatorClient | undefined

export function activate(context: vscode.ExtensionContext) {
  console.log('Hermes Game Operator extension is now active')

  // Initialize the client
  client = new GameOperatorClient()

  // Register tree view providers
  const taskTreeProvider = new TaskTreeProvider(client)
  const timelineViewProvider = new TimelineViewProvider(client)
  const approvalsViewProvider = new ApprovalsViewProvider(client)

  vscode.window.registerTreeDataProvider('gameOperator.tasks', taskTreeProvider)
  vscode.window.registerTreeDataProvider('gameOperator.timeline', timelineViewProvider)
  vscode.window.registerTreeDataProvider('gameOperator.approvals', approvalsViewProvider)

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand('gameOperator.connect', async () => {
      const config = vscode.workspace.getConfiguration('gameOperator')
      const serverUrl = config.get<string>('serverUrl', 'http://127.0.0.1:1422')
      const authToken = config.get<string>('authToken')

      if (!serverUrl) {
        vscode.window.showErrorMessage('Please configure gameOperator.serverUrl')
        return
      }

      try {
        await client?.connect(serverUrl, authToken)
        vscode.window.showInformationMessage('Connected to Game Operator')
        taskTreeProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to connect: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.disconnect', async () => {
      await client?.disconnect()
      vscode.window.showInformationMessage('Disconnected from Game Operator')
      taskTreeProvider.refresh()
    }),

    vscode.commands.registerCommand('gameOperator.startTask', async () => {
      const goal = await vscode.window.showInputBox({
        prompt: 'Enter task goal',
        placeHolder: 'e.g., Add double jump to the player character'
      })

      if (!goal) return

      const projectPath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
      if (!projectPath) {
        vscode.window.showErrorMessage('Open a Godot project folder before starting a task')
        return
      }

      try {
        const task = await client?.startTask(goal, projectPath)
        vscode.window.showInformationMessage(`Task started: ${task?.task_id}`)
        taskTreeProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to start task: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.pauseTask', async (task) => {
      try {
        await client?.pauseTask(task.task_id)
        vscode.window.showInformationMessage('Task paused')
        taskTreeProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to pause task: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.resumeTask', async (task) => {
      try {
        await client?.resumeTask(task.task_id)
        vscode.window.showInformationMessage('Task resumed')
        taskTreeProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to resume task: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.stopTask', async (task) => {
      const confirm = await vscode.window.showWarningMessage(
        'Are you sure you want to stop this task?',
        'Yes',
        'No'
      )

      if (confirm !== 'Yes') return

      try {
        await client?.stopTask(task.task_id)
        vscode.window.showInformationMessage('Task stopped')
        taskTreeProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to stop task: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.approve', async (approval) => {
      try {
        await client?.approve(approval.task_id, approval.approval_id, 'approve')
        vscode.window.showInformationMessage('Approved')
        approvalsViewProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to approve: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.reject', async (approval) => {
      try {
        await client?.approve(approval.task_id, approval.approval_id, 'reject')
        vscode.window.showInformationMessage('Rejected')
        approvalsViewProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to reject: ${error.message}`)
      }
    }),

    vscode.commands.registerCommand('gameOperator.showApproval', async (approval) => {
      const decision = await vscode.window.showInformationMessage(
        `${approval.title}\n${approval.reason}`,
        'Approve',
        'Reject'
      )
      if (!decision) return

      try {
        await client?.approve(
          approval.task_id,
          approval.approval_id,
          decision === 'Approve' ? 'approve' : 'reject'
        )
        vscode.window.showInformationMessage(decision === 'Approve' ? 'Approved' : 'Rejected')
        approvalsViewProvider.refresh()
      } catch (error: any) {
        vscode.window.showErrorMessage(`Failed to resolve approval: ${error.message}`)
      }
    })
  )

  // Auto-connect if configured
  const config = vscode.workspace.getConfiguration('gameOperator')
  if (config.get<string>('serverUrl')) {
    client.connect(
      config.get<string>('serverUrl')!,
      config.get<string>('authToken')
    ).then(() => {
      taskTreeProvider.refresh()
    }).catch(() => {
      // Silent fail on auto-connect
    })
  }
}

export function deactivate() {
  client?.disconnect()
}

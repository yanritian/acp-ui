package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages
import com.github.yfanitian.gameoperator.ui.GameOperatorPanelRegistry

class ConnectAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        GameOperatorPanelRegistry.get(project)?.connect()
            ?: Messages.showInfoMessage(project, "Open the Game Operator tool window first.", "Game Operator")
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

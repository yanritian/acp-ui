package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages

class ConnectAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        Messages.showInfoMessage(e.project, "Connecting to Game Operator...", "Game Operator")
        // TODO: Implement actual connection
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}
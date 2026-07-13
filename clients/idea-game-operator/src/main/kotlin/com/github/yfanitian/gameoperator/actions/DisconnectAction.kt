package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.github.yfanitian.gameoperator.ui.GameOperatorPanelRegistry

class DisconnectAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        e.project?.let { GameOperatorPanelRegistry.get(it)?.disconnect() }
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

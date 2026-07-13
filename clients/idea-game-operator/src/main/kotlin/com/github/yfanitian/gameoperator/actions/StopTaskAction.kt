package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.github.yfanitian.gameoperator.ui.GameOperatorPanelRegistry

class StopTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        e.project?.let { GameOperatorPanelRegistry.get(it)?.stopTask() }
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

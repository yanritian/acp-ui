package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.github.yfanitian.gameoperator.ui.GameOperatorPanelRegistry

class ResumeTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        e.project?.let { GameOperatorPanelRegistry.get(it)?.resumeTask() }
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

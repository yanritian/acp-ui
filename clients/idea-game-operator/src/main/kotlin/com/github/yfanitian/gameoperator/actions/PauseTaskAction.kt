package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent

class PauseTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        // TODO: Implement actual pause
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

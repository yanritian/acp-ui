package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent

class ResumeTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        // TODO: Implement actual resume
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

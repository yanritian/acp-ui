package com.github.yfanitian.gameoperator.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages

class StartTaskAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val goal = Messages.showInputDialog(e.project, "Enter task goal:", "Start Task", null)
        if (goal != null) {
            // TODO: Implement actual task creation
        }
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabledAndVisible = e.project != null
    }
}

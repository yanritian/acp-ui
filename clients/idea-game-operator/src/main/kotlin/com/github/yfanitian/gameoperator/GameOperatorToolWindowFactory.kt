package com.github.yfanitian.gameoperator

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.github.yfanitian.gameoperator.ui.GameOperatorPanel

class GameOperatorToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val gameOperatorPanel = GameOperatorPanel(project)
        val contentFactory = ContentFactory.getInstance()
        val content = contentFactory.createContent(gameOperatorPanel.getContent(), "", false)
        toolWindow.contentManager.addContent(content)
    }

    override fun shouldBeAvailable(project: Project) = true
}
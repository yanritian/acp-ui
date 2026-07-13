package com.github.yfanitian.gameoperator.ui

import com.intellij.openapi.project.Project
import java.util.WeakHashMap

/** Keeps the tool-window panel available to actions registered in plugin.xml. */
object GameOperatorPanelRegistry {
    private val panels = WeakHashMap<Project, GameOperatorPanel>()

    @Synchronized
    fun register(project: Project, panel: GameOperatorPanel) {
        panels[project] = panel
    }

    @Synchronized
    fun get(project: Project): GameOperatorPanel? = panels[project]

    @Synchronized
    fun unregister(project: Project) {
        panels.remove(project)
    }
}

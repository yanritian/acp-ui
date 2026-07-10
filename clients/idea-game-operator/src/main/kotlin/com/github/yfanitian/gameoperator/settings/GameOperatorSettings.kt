package com.github.yfanitian.gameoperator.settings

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.util.xmlb.XmlSerializerUtil

@State(
    name = "com.github.yfanitian.gameoperator.settings.GameOperatorSettings",
    storages = [Storage("GameOperatorSettings.xml")]
)
class GameOperatorSettings : PersistentStateComponent<GameOperatorSettings> {
    var serverUrl: String = "http://localhost:8080"
    var authToken: String = ""

    override fun getState(): GameOperatorSettings = this

    override fun loadState(state: GameOperatorSettings) {
        XmlSerializerUtil.copyBean(state, this)
    }

    companion object {
        fun getInstance(): GameOperatorSettings {
            return ApplicationManager.getApplication().getService(GameOperatorSettings::class.java)
        }
    }
}

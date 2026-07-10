package com.github.yfanitian.gameoperator.settings

import com.intellij.openapi.options.Configurable
import javax.swing.*
import java.awt.BorderLayout

class GameOperatorConfigurable : Configurable {
    private var mainPanel: JPanel? = null
    private var serverUrlField: JTextField? = null
    private var authTokenField: JPasswordField? = null

    override fun getDisplayName(): String = "Hermes Game Operator"

    override fun createComponent(): JComponent {
        mainPanel = JPanel(BorderLayout())

        val formPanel = JPanel()
        formPanel.layout = BoxLayout(formPanel, BoxLayout.Y_AXIS)

        serverUrlField = JTextField()
        authTokenField = JPasswordField()

        formPanel.add(JLabel("Server URL:"))
        formPanel.add(serverUrlField)
        formPanel.add(Box.createVerticalStrut(10))
        formPanel.add(JLabel("Auth Token:"))
        formPanel.add(authTokenField)

        mainPanel!!.add(formPanel, BorderLayout.NORTH)

        return mainPanel!!
    }

    override fun isModified(): Boolean {
        val settings = GameOperatorSettings.getInstance()
        return serverUrlField?.text != settings.serverUrl ||
               String(authTokenField?.password ?: charArrayOf(0)) != settings.authToken
    }

    override fun apply() {
        val settings = GameOperatorSettings.getInstance()
        settings.serverUrl = serverUrlField?.text ?: ""
        settings.authToken = String(authTokenField?.password ?: charArrayOf(0))
    }

    override fun reset() {
        val settings = GameOperatorSettings.getInstance()
        serverUrlField?.text = settings.serverUrl
        authTokenField?.text = settings.authToken
    }

    override fun disposeUIResources() {
        mainPanel = null
        serverUrlField = null
        authTokenField = null
    }
}

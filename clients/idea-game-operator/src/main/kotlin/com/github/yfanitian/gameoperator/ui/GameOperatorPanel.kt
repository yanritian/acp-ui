package com.github.yfanitian.gameoperator.ui

import com.intellij.openapi.project.Project
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.table.JBTable
import java.awt.BorderLayout
import java.awt.FlowLayout
import javax.swing.*
import javax.swing.table.DefaultTableModel

class GameOperatorPanel(private val project: Project) {
    private val mainPanel = JPanel(BorderLayout())
    private val statusLabel = JBLabel("Status: Not Connected")
    private val taskTable = JBTable(DefaultTableModel(arrayOf("ID", "Goal", "Status"), 0))
    private val eventList = DefaultListModel<String>()

    init {
        setupUI()
    }

    private fun setupUI() {
        // Top panel with status
        val topPanel = JPanel(FlowLayout(FlowLayout.LEFT))
        topPanel.add(statusLabel)
        mainPanel.add(topPanel, BorderLayout.NORTH)

        // Center panel with tabs
        val tabbedPane = JTabbedPane()

        // Tasks tab
        val tasksPanel = JPanel(BorderLayout())
        tasksPanel.add(JBScrollPane(taskTable), BorderLayout.CENTER)
        tabbedPane.addTab("Tasks", tasksPanel)

        // Events tab
        val eventsPanel = JPanel(BorderLayout())
        val eventJList = JList(eventList)
        eventsPanel.add(JBScrollPane(eventJList), BorderLayout.CENTER)
        tabbedPane.addTab("Events", eventsPanel)

        // Approvals tab
        val approvalsPanel = JPanel(BorderLayout())
        approvalsPanel.add(JBLabel("No pending approvals"), BorderLayout.CENTER)
        tabbedPane.addTab("Approvals", approvalsPanel)

        mainPanel.add(tabbedPane, BorderLayout.CENTER)

        // Bottom panel with buttons
        val bottomPanel = JPanel(FlowLayout(FlowLayout.LEFT))
        bottomPanel.add(JButton("Connect").apply {
            addActionListener { connect() }
        })
        bottomPanel.add(JButton("Start Task").apply {
            addActionListener { startTask() }
        })
        bottomPanel.add(JButton("Pause").apply {
            addActionListener { pauseTask() }
        })
        bottomPanel.add(JButton("Resume").apply {
            addActionListener { resumeTask() }
        })
        bottomPanel.add(JButton("Stop").apply {
            addActionListener { stopTask() }
        })
        mainPanel.add(bottomPanel, BorderLayout.SOUTH)
    }

    fun getContent(): JComponent = mainPanel

    private fun connect() {
        statusLabel.text = "Status: Connecting..."
        // TODO: Implement actual connection
        statusLabel.text = "Status: Connected"
        statusLabel.foreground = JBColor.GREEN
    }

    private fun startTask() {
        val goal = JOptionPane.showInputDialog(mainPanel, "Enter task goal:")
        if (goal != null) {
            // TODO: Implement actual task creation
            eventList.addElement("Task started: $goal")
        }
    }

    private fun pauseTask() {
        // TODO: Implement actual pause
        eventList.addElement("Task paused")
    }

    private fun resumeTask() {
        // TODO: Implement actual resume
        eventList.addElement("Task resumed")
    }

    private fun stopTask() {
        // TODO: Implement actual stop
        eventList.addElement("Task stopped")
    }
}
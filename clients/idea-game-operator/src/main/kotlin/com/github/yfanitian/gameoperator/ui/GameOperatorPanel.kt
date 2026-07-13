package com.github.yfanitian.gameoperator.ui

import com.github.yfanitian.gameoperator.api.ApprovalRequest
import com.github.yfanitian.gameoperator.api.GameOperatorApiClient
import com.github.yfanitian.gameoperator.api.OperatorEvent
import com.github.yfanitian.gameoperator.api.OperatorTask
import com.github.yfanitian.gameoperator.settings.GameOperatorSettings
import com.intellij.openapi.Disposable
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.table.JBTable
import java.awt.BorderLayout
import java.awt.FlowLayout
import java.util.concurrent.ExecutionException
import javax.swing.*
import javax.swing.table.DefaultTableModel

class GameOperatorPanel(private val project: Project) : Disposable {
    private val mainPanel = JPanel(BorderLayout())
    private val statusLabel = JBLabel("Status: Not Connected")
    private val taskModel = DefaultTableModel(arrayOf("ID", "Goal", "Status"), 0)
    private val taskTable = JBTable(taskModel)
    private val eventList = DefaultListModel<String>()
    private val approvalModel = DefaultListModel<String>()
    private val approvals = mutableListOf<ApprovalRequest>()
    private var apiClient: GameOperatorApiClient? = null
    private var activeTaskId: String? = null

    init {
        GameOperatorPanelRegistry.register(project, this)
        setupUI()
    }

    private fun setupUI() {
        val topPanel = JPanel(FlowLayout(FlowLayout.LEFT))
        topPanel.add(statusLabel)
        mainPanel.add(topPanel, BorderLayout.NORTH)

        val tabbedPane = JTabbedPane()

        val tasksPanel = JPanel(BorderLayout())
        taskTable.selectionModel.addListSelectionListener {
            if (!it.valueIsAdjusting) {
                activeTaskId = selectedTaskId()
                refreshTaskDetails()
            }
        }
        tasksPanel.add(JBScrollPane(taskTable), BorderLayout.CENTER)
        tabbedPane.addTab("Tasks", tasksPanel)

        val eventsPanel = JPanel(BorderLayout())
        eventsPanel.add(JBScrollPane(JList(eventList)), BorderLayout.CENTER)
        tabbedPane.addTab("Events", eventsPanel)

        val approvalsPanel = JPanel(BorderLayout())
        val approvalList = JList(approvalModel)
        approvalList.selectionModel.addListSelectionListener {
            if (!it.valueIsAdjusting) {
                val approval = approvals.getOrNull(approvalList.selectedIndex)
                if (approval != null) {
                    val decision = Messages.showYesNoDialog(
                        project,
                        "${approval.title}\n\n${approval.reason}",
                        "Approval Required",
                        "Approve",
                        "Reject",
                        null
                    )
                    resolveApproval(approval, if (decision == Messages.YES) "approve" else "reject")
                }
            }
        }
        approvalsPanel.add(JBScrollPane(approvalList), BorderLayout.CENTER)
        tabbedPane.addTab("Approvals", approvalsPanel)

        mainPanel.add(tabbedPane, BorderLayout.CENTER)

        val bottomPanel = JPanel(FlowLayout(FlowLayout.LEFT))
        bottomPanel.add(JButton("Connect").apply { addActionListener { connect() } })
        bottomPanel.add(JButton("Disconnect").apply { addActionListener { disconnect() } })
        bottomPanel.add(JButton("Start Task").apply { addActionListener { startTask() } })
        bottomPanel.add(JButton("Pause").apply { addActionListener { pauseTask() } })
        bottomPanel.add(JButton("Resume").apply { addActionListener { resumeTask() } })
        bottomPanel.add(JButton("Stop").apply { addActionListener { stopTask() } })
        mainPanel.add(bottomPanel, BorderLayout.SOUTH)
    }

    fun getContent(): JComponent = mainPanel

    fun connect() {
        val settings = GameOperatorSettings.getInstance()
        val url = settings.serverUrl.trim().trimEnd('/')
        if (url.isEmpty()) {
            showError("Server URL is empty")
            return
        }

        statusLabel.text = "Status: Connecting..."
        runAsync(
            work = {
                val client = GameOperatorApiClient(url, settings.authToken.ifBlank { null })
                if (!client.isConnected()) throw IllegalStateException("Game Operator server is unreachable")
                client
            },
            onSuccess = { client ->
                apiClient = client
                statusLabel.text = "Status: Connected"
                statusLabel.foreground = JBColor.GREEN
                refreshTasks()
            }
        )
    }

    fun disconnect() {
        apiClient = null
        activeTaskId = null
        taskModel.setRowCount(0)
        eventList.clear()
        approvals.clear()
        approvalModel.clear()
        statusLabel.text = "Status: Not Connected"
        statusLabel.foreground = JBColor.foreground()
    }

    fun startTask() {
        val goal = Messages.showInputDialog(mainPanel, "Enter task goal:", "Start Task", null) ?: return
        if (goal.isBlank()) return
        val client = requireClient() ?: return
        runAsync(
            work = { client.startTask(goal, project.basePath) },
            onSuccess = { task ->
                activeTaskId = task.task_id
                refreshTasks()
            }
        )
    }

    fun pauseTask() = runTaskCommand("pause") { client, taskId -> client.pauseTask(taskId) }

    fun resumeTask() = runTaskCommand("resume") { client, taskId -> client.resumeTask(taskId) }

    fun stopTask() = runTaskCommand("stop") { client, taskId -> client.stopTask(taskId) }

    fun refreshTasks() {
        val client = apiClient ?: return
        runAsync(
            work = { client.listTasks() },
            onSuccess = { tasks ->
                taskModel.setRowCount(0)
                tasks.forEach { task ->
                    taskModel.addRow(arrayOf<Any>(task.task_id, task.goal, task.status))
                }
                if (activeTaskId == null) activeTaskId = tasks.firstOrNull()?.task_id
                refreshTaskDetails()
            }
        )
    }

    private fun refreshTaskDetails() {
        val client = apiClient ?: return
        val taskId = activeTaskId ?: return
        runAsync(
            work = {
                TaskDetails(
                    events = client.listEvents(taskId),
                    approvals = client.getPendingApprovals(taskId)
                )
            },
            onSuccess = { details ->
                eventList.clear()
                details.events.forEach { event -> eventList.addElement(formatEvent(event)) }
                approvals.clear()
                approvals.addAll(details.approvals)
                approvalModel.clear()
                approvals.forEach { approvalModel.addElement("${it.level}: ${it.title}") }
            }
        )
    }

    private fun resolveApproval(approval: ApprovalRequest, decision: String) {
        val client = requireClient() ?: return
        runAsync(
            work = { client.approve(approval.task_id, approval.approval_id, decision) },
            onSuccess = { refreshTaskDetails() }
        )
    }

    private fun runTaskCommand(
        action: String,
        command: (GameOperatorApiClient, String) -> Unit
    ) {
        val client = requireClient() ?: return
        val taskId = activeTaskId ?: selectedTaskId()
        if (taskId == null) {
            showError("Select a task first")
            return
        }
        runAsync(
            work = { command(client, taskId) },
            onSuccess = {
                eventList.addElement("Task $action requested: $taskId")
                refreshTasks()
            }
        )
    }

    private fun selectedTaskId(): String? {
        val row = taskTable.selectedRow
        return if (row >= 0) taskModel.getValueAt(row, 0).toString() else activeTaskId
    }

    private fun requireClient(): GameOperatorApiClient? {
        if (apiClient == null) showError("Connect to Game Operator first")
        return apiClient
    }

    private fun formatEvent(event: OperatorEvent): String =
        "[${event.level}] ${event.title}: ${event.message.orEmpty()}"

    private fun showError(message: String) {
        Messages.showErrorDialog(mainPanel, message, "Game Operator")
        statusLabel.text = "Status: Error"
        statusLabel.foreground = JBColor.RED
    }

    private fun <T> runAsync(work: () -> T, onSuccess: (T) -> Unit) {
        object : SwingWorker<T, Unit>() {
            override fun doInBackground(): T = work()

            override fun done() {
                try {
                    onSuccess(get())
                } catch (error: InterruptedException) {
                    Thread.currentThread().interrupt()
                    showError("Operation interrupted")
                } catch (error: ExecutionException) {
                    showError(error.cause?.message ?: "Operation failed")
                }
            }
        }.execute()
    }

    override fun dispose() {
        GameOperatorPanelRegistry.unregister(project)
        disconnect()
    }

    private data class TaskDetails(
        val events: List<OperatorEvent>,
        val approvals: List<ApprovalRequest>
    )
}

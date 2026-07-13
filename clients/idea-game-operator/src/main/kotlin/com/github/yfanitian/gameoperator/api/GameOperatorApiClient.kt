package com.github.yfanitian.gameoperator.api

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.TimeUnit

data class OperatorTask(
    val task_id: String,
    /** Optimistic concurrency control revision, incremented on every state change */
    val revision: Long,
    val domain: String,
    val project_path: String,
    val goal: String,
    val status: String,
    val mode: String,
    val approval_policy: String,
    val created_at: String,
    val updated_at: String,
    val checkpoint_id: String?,
    val memory_snapshot_id: String?
)

data class OperatorEvent(
    val event_id: String,
    val task_id: String,
    val sequence: Long,
    val task_revision: Long,
    val timestamp: String,
    val type: String,
    val level: String,
    val title: String,
    val message: String?,
    val source: String
)

data class ApprovalRequest(
    val approval_id: String,
    val task_id: String,
    val task_revision: Long,
    val level: String,
    val action: String,
    val title: String,
    val reason: String,
    val risk: String?
)

data class TaskListResponse(
    val tasks: List<OperatorTask>
)

data class EventListResponse(
    val events: List<OperatorEvent>
)

data class ApprovalListResponse(
    val approvals: List<ApprovalRequest>
)

data class StartTaskResponse(
    val task_id: String,
    val revision: Long,
    val status: String,
    val event_stream: String
)

class GameOperatorApiClient(
    private val baseUrl: String,
    private val authToken: String? = null
) {
    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .writeTimeout(30, TimeUnit.SECONDS)
        .build()

    private val gson = Gson()
    private val JSON = "application/json; charset=utf-8".toMediaType()

    fun isConnected(): Boolean {
        return try {
            val request = Request.Builder()
                .url("$baseUrl/api/health")
                .get()
                .apply {
                    authToken?.let { addHeader("Authorization", "Bearer $it") }
                }
                .build()

            client.newCall(request).execute().use { response ->
                response.isSuccessful
            }
        } catch (e: Exception) {
            false
        }
    }

    @Throws(IOException::class)
    fun listTasks(): List<OperatorTask> {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks")
            .get()
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to list tasks: ${response.code}")
            }

            val body = response.body?.string() ?: throw IOException("Empty response")

            return try {
                val taskListResponse = gson.fromJson(body, TaskListResponse::class.java)
                taskListResponse.tasks
            } catch (e: Exception) {
                // Try parsing as array directly
                gson.fromJson(body, Array<OperatorTask>::class.java).toList()
            }
        }
    }

    @Throws(IOException::class)
    fun getTask(taskId: String): OperatorTask {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks/$taskId")
            .get()
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to get task: ${response.code}")
            }

            val body = response.body?.string() ?: throw IOException("Empty response")
            return gson.fromJson(body, OperatorTask::class.java)
        }
    }

    @Throws(IOException::class)
    fun startTask(goal: String, projectPath: String? = null): OperatorTask {
        val json = gson.toJson(mapOf(
            "domain" to "game.godot",
            "project_path" to (projectPath ?: ""),
            "goal" to goal,
            "mode" to "propose_then_apply",
            "approval_policy" to "safe_default"
        ))

        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks")
            .post(json.toRequestBody(JSON))
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to start task: ${response.code}")
            }

            val body = response.body?.string() ?: throw IOException("Empty response")
            val startResponse = gson.fromJson(body, StartTaskResponse::class.java)
            return getTask(startResponse.task_id)
        }
    }

    @Throws(IOException::class)
    fun pauseTask(taskId: String, expectedRevision: Long? = null) {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks/$taskId/pause")
            .post(gson.toJson(expectedRevision?.let { mapOf("expected_revision" to it) } ?: emptyMap<String, Any>()).toRequestBody(JSON))
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to pause task: ${response.code}")
            }
        }
    }

    @Throws(IOException::class)
    fun resumeTask(taskId: String, expectedRevision: Long? = null) {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks/$taskId/resume")
            .post(gson.toJson(expectedRevision?.let { mapOf("expected_revision" to it) } ?: emptyMap<String, Any>()).toRequestBody(JSON))
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to resume task: ${response.code}")
            }
        }
    }

    @Throws(IOException::class)
    fun stopTask(taskId: String, expectedRevision: Long? = null) {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks/$taskId/stop")
            .post(gson.toJson(expectedRevision?.let { mapOf("expected_revision" to it) } ?: emptyMap<String, Any>()).toRequestBody(JSON))
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to stop task: ${response.code}")
            }
        }
    }

    @Throws(IOException::class)
    fun listEvents(taskId: String): List<OperatorEvent> {
        val request = Request.Builder()
            .url("$baseUrl/api/operator/tasks/$taskId/events")
            .get()
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to list events: ${response.code}")
            }

            val body = response.body?.string() ?: throw IOException("Empty response")

            return try {
                val eventListResponse = gson.fromJson(body, EventListResponse::class.java)
                eventListResponse.events
            } catch (e: Exception) {
                gson.fromJson(body, Array<OperatorEvent>::class.java).toList()
            }
        }
    }

    @Throws(IOException::class)
    fun getPendingApprovals(taskId: String? = null): List<ApprovalRequest> {
        if (taskId == null) {
            return listTasks().flatMap { task -> getPendingApprovals(task.task_id) }
        }

        val url = "$baseUrl/api/operator/tasks/$taskId/approvals"

        val request = Request.Builder()
            .url(url)
            .get()
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to get approvals: ${response.code}")
            }

            val body = response.body?.string() ?: throw IOException("Empty response")

            return try {
                val approvalListResponse = gson.fromJson(body, ApprovalListResponse::class.java)
                approvalListResponse.approvals
            } catch (e: Exception) {
                gson.fromJson(body, Array<ApprovalRequest>::class.java).toList()
            }
        }
    }

    @Throws(IOException::class)
    fun approve(taskId: String, approvalId: String, decision: String, expectedRevision: Long? = null) {
        val json = gson.toJson(
            mapOf(
                "task_id" to taskId,
                "approval_id" to approvalId,
                "decision" to decision,
                "expected_revision" to expectedRevision
            )
        )

        val request = Request.Builder()
            .url("$baseUrl/api/operator/approvals/decision")
            .post(json.toRequestBody(JSON))
            .apply {
                authToken?.let { addHeader("Authorization", "Bearer $it") }
            }
            .build()

        client.newCall(request).execute().use { response ->
            if (!response.isSuccessful) {
                throw IOException("Failed to approve: ${response.code}")
            }
        }
    }
}

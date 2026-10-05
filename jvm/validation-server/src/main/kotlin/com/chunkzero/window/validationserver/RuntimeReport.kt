package com.chunkzero.window.validationserver

import dev.rpp.mcvalidation.minestom.ValidationEventSink
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption

@Serializable
internal data class ReportEvent(
    val sequence: Int,
    val name: String,
    val details: Map<String, String>,
)

@Serializable
internal data class ReportDocument(
    @SerialName("schema_version") val schemaVersion: Int = 1,
    val target: String = "window-minestom-runtime",
    val events: List<ReportEvent>,
)

internal class RuntimeReport(
    private val path: Path,
) {
    private val events = mutableListOf<ReportEvent>()
    private var validationEvents: ValidationEventSink? = null
    private val json =
        Json {
            prettyPrint = true
            encodeDefaults = true
        }

    @Synchronized
    fun record(
        name: String,
        details: Map<String, String>,
    ) {
        events += ReportEvent(events.size + 1, name, details.toSortedMap())
        validationEvents?.emit(name, details)
        Files.createDirectories(path.parent)
        val temporary = path.resolveSibling("${path.fileName}.tmp")
        Files.writeString(temporary, json.encodeToString(ReportDocument(events = events)))
        Files.move(
            temporary,
            path,
            StandardCopyOption.REPLACE_EXISTING,
            StandardCopyOption.ATOMIC_MOVE,
        )
    }

    fun attachValidationEvents(events: ValidationEventSink) {
        validationEvents = events
    }
}

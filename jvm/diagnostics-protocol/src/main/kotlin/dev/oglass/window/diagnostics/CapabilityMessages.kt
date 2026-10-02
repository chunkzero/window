package dev.oglass.window.diagnostics

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Client-to-server opt-in. No frames are sent until this message is accepted. */
@Serializable
public data class CapabilityHello
    @JvmOverloads
    constructor(
        @SerialName("schema_version")
        public val schemaVersion: Int = DiagnosticsProtocol.SCHEMA_VERSION,
        @SerialName("minimum_schema_version") public val minimumSchemaVersion: Int = schemaVersion,
        @SerialName("maximum_schema_version") public val maximumSchemaVersion: Int = schemaVersion,
        @SerialName("client_nonce") public val clientNonce: String,
        @SerialName("max_payload_bytes")
        public val maxPayloadBytes: Int = DiagnosticsProtocol.MAX_PAYLOAD_BYTES,
        public val features: List<String> = listOf(DiagnosticsProtocol.RENDER_FRAME_FEATURE),
    ) {
        internal fun validate() {
            require(schemaVersion == DiagnosticsProtocol.SCHEMA_VERSION) {
                "Unsupported capability hello schema version $schemaVersion"
            }
            require(minimumSchemaVersion in 1..maximumSchemaVersion) { "Invalid schema version range" }
            require(clientNonce.length in 1..128) { "Client nonce must contain 1..128 characters" }
            require(maxPayloadBytes in 1024..DiagnosticsProtocol.MAX_PAYLOAD_BYTES) {
                "Invalid requested payload limit $maxPayloadBytes"
            }
            require(features.size <= 32 && features.all { it.length in 1..64 }) {
                "Capability features exceed protocol limits"
            }
        }
    }

/** Server-to-client result for one [CapabilityHello]. */
@Serializable
public data class CapabilityAck
    @JvmOverloads
    constructor(
        @SerialName("schema_version")
        public val schemaVersion: Int = DiagnosticsProtocol.SCHEMA_VERSION,
        public val accepted: Boolean,
        @SerialName("client_nonce") public val clientNonce: String,
        @SerialName("session_id") public val sessionId: String? = null,
        @SerialName("max_payload_bytes") public val maxPayloadBytes: Int? = null,
        public val features: List<String> = emptyList(),
        public val reason: String? = null,
    ) {
        internal fun validate() {
            require(schemaVersion == DiagnosticsProtocol.SCHEMA_VERSION) {
                "Unsupported capability ack schema version $schemaVersion"
            }
            require(clientNonce.length in 1..128) { "Client nonce must contain 1..128 characters" }
            require(reason == null || reason.length <= 256) { "Capability reason is too long" }
            require(features.size <= 32 && features.all { it.length in 1..64 }) {
                "Capability features exceed protocol limits"
            }
            if (accepted) {
                require(!sessionId.isNullOrBlank() && sessionId.length <= 128) {
                    "Accepted capability ack requires a bounded session id"
                }
                require(maxPayloadBytes in 1024..DiagnosticsProtocol.MAX_PAYLOAD_BYTES) {
                    "Accepted capability ack requires a valid payload limit"
                }
            }
        }
    }

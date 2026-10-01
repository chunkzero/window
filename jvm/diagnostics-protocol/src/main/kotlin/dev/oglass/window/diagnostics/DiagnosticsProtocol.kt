package dev.oglass.window.diagnostics

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction
import java.nio.charset.StandardCharsets

/** Wire-level constants shared by Window diagnostics clients and servers. */
public object DiagnosticsProtocol {
    public const val SCHEMA_VERSION: Int = 1
    public const val CAPABILITY_CHANNEL: String = "window:diagnostics/capability"
    public const val FRAME_CHANNEL: String = "window:diagnostics/frame"
    public const val RENDER_FRAME_FEATURE: String = "render_frame_v1"
    public const val MAX_PAYLOAD_BYTES: Int = 256 * 1024

    private val json =
        Json {
            encodeDefaults = true
            explicitNulls = false
            ignoreUnknownKeys = true
            isLenient = false
        }

    /** Encodes a capability hello, rejecting output beyond [maxBytes]. */
    @JvmStatic
    @JvmOverloads
    public fun encodeHello(
        message: CapabilityHello,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): ByteArray {
        message.validate()
        return encode(message, CapabilityHello.serializer(), maxBytes)
    }

    /** Decodes and validates a capability hello from untrusted bytes. */
    @JvmStatic
    @JvmOverloads
    public fun decodeHello(
        bytes: ByteArray,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): CapabilityHello = decode(bytes, CapabilityHello.serializer(), maxBytes).also(CapabilityHello::validate)

    /** Encodes a capability acknowledgement, rejecting output beyond [maxBytes]. */
    @JvmStatic
    @JvmOverloads
    public fun encodeAck(
        message: CapabilityAck,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): ByteArray {
        message.validate()
        return encode(message, CapabilityAck.serializer(), maxBytes)
    }

    /** Decodes and validates a capability acknowledgement from untrusted bytes. */
    @JvmStatic
    @JvmOverloads
    public fun decodeAck(
        bytes: ByteArray,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): CapabilityAck = decode(bytes, CapabilityAck.serializer(), maxBytes).also(CapabilityAck::validate)

    /** Encodes a render frame, rejecting output beyond [maxBytes]. */
    @JvmStatic
    @JvmOverloads
    public fun encodeFrame(
        frame: RenderFrame,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): ByteArray {
        frame.validate()
        return encode(frame, RenderFrame.serializer(), maxBytes)
    }

    /** Decodes and validates a render frame from untrusted bytes. */
    @JvmStatic
    @JvmOverloads
    public fun decodeFrame(
        bytes: ByteArray,
        maxBytes: Int = MAX_PAYLOAD_BYTES,
    ): RenderFrame = decode(bytes, RenderFrame.serializer(), maxBytes).also(RenderFrame::validate)

    private fun <T> encode(
        value: T,
        serializer: kotlinx.serialization.KSerializer<T>,
        maxBytes: Int,
    ): ByteArray {
        requireValidLimit(maxBytes)
        val bytes = json.encodeToString(serializer, value).toByteArray(StandardCharsets.UTF_8)
        require(bytes.size <= maxBytes) {
            "Window diagnostics payload is ${bytes.size} bytes; limit is $maxBytes"
        }
        return bytes
    }

    private fun <T> decode(
        bytes: ByteArray,
        serializer: kotlinx.serialization.KSerializer<T>,
        maxBytes: Int,
    ): T {
        requireValidLimit(maxBytes)
        require(bytes.size <= maxBytes) {
            "Window diagnostics payload is ${bytes.size} bytes; limit is $maxBytes"
        }
        val decoder =
            StandardCharsets.UTF_8
                .newDecoder()
                .onMalformedInput(CodingErrorAction.REPORT)
                .onUnmappableCharacter(CodingErrorAction.REPORT)
        val text =
            try {
                decoder.decode(ByteBuffer.wrap(bytes)).toString()
            } catch (error: java.nio.charset.CharacterCodingException) {
                throw SerializationException("Window diagnostics payload is not valid UTF-8", error)
            }
        return json.decodeFromString(serializer, text)
    }

    private fun requireValidLimit(maxBytes: Int) {
        require(maxBytes in 1..MAX_PAYLOAD_BYTES) {
            "Window diagnostics payload limit must be between 1 and $MAX_PAYLOAD_BYTES"
        }
    }
}

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

/** Expected render state produced by the real Window server composition pass. */
@Serializable
public data class RenderFrame(
    @SerialName("schema_version")
    public val schemaVersion: Int = DiagnosticsProtocol.SCHEMA_VERSION,
    @SerialName("frame_id") public val frameId: Long,
    public val reason: RenderFrameReason,
    public val correlation: RenderCorrelation,
    @SerialName("cursor_convention") public val cursorConvention: RenderCursorConvention,
    @SerialName("cursor_start") public val cursorStart: Int,
    @SerialName("cursor_end") public val cursorEnd: Int,
    @SerialName("net_cursor_delta") public val netCursorDelta: Int,
    @SerialName("pack_fingerprint") public val packFingerprint: PackFingerprint? = null,
    public val layers: List<RenderLayerTrace>,
) {
    internal fun validate() {
        require(schemaVersion == DiagnosticsProtocol.SCHEMA_VERSION) {
            "Unsupported render frame schema version $schemaVersion"
        }
        require(frameId > 0) { "Frame id must be positive" }
        correlation.validate()
        require(netCursorDelta == cursorEnd - cursorStart) { "Invalid frame net cursor delta" }
        packFingerprint?.validate()
        require(layers.size <= 4096) { "Render frame contains too many layers" }
        require(layers.map(RenderLayerTrace::semanticId).toSet().size == layers.size) {
            "Render frame semantic layer ids must be unique"
        }
        layers.forEach(RenderLayerTrace::validate)
    }
}

/** How semantic layer segments participate in a frame's cursor reset. */
@Serializable
public enum class RenderCursorConvention {
    /** Every layer contains its own leading and trailing reset to the surface origin. */
    @SerialName("independent_net_zero_segments")
    INDEPENDENT_NET_ZERO_SEGMENTS,

    /** Layers share a cursor and the completed frame performs one reset to its fixed width. */
    @SerialName("fixed_width_composition")
    FIXED_WIDTH_COMPOSITION,
}

@Serializable
public enum class RenderFrameReason {
    @SerialName("open")
    OPEN,

    @SerialName("reactive_update")
    REACTIVE_UPDATE,
}

@Serializable
public enum class RenderSurfaceKind {
    @SerialName("window")
    WINDOW,

    @SerialName("hud")
    HUD,
}

/** Robust key joining a frame to a concrete server session and client surface. */
@Serializable
public data class RenderCorrelation(
    @SerialName("surface_kind") public val surfaceKind: RenderSurfaceKind,
    @SerialName("semantic_id") public val semanticId: String,
    @SerialName("render_session_id") public val renderSessionId: String,
    @SerialName("container_id") public val containerId: Int? = null,
    @SerialName("hud_channel") public val hudChannel: String? = null,
) {
    internal fun validate() {
        require(semanticId.length in 1..256) { "Invalid semantic id" }
        require(renderSessionId.length in 1..128) { "Invalid render session id" }
        require(containerId == null || containerId in 0..255) { "Invalid container id" }
        require(hudChannel == null || hudChannel.length in 1..64) { "Invalid HUD channel" }
    }
}

@Serializable
public data class PackFingerprint(
    public val algorithm: String,
    public val value: String,
) {
    internal fun validate() {
        require(algorithm == "sha256") { "Unsupported pack fingerprint algorithm '$algorithm'" }
        require(value.length == 64 && value.all { it in '0'..'9' || it in 'a'..'f' }) {
            "Pack fingerprint must be 64 lowercase hexadecimal characters"
        }
    }
}

@Serializable
public enum class RenderLayerKind {
    @SerialName("static_chrome")
    STATIC_CHROME,

    @SerialName("text_slot")
    TEXT_SLOT,

    @SerialName("sprite_slot")
    SPRITE_SLOT,

    @SerialName("hud_static")
    HUD_STATIC,

    @SerialName("hud_text")
    HUD_TEXT,
}

/** One semantically-addressable server expectation within a [RenderFrame]. */
@Serializable
public data class RenderLayerTrace(
    @SerialName("semantic_id") public val semanticId: String,
    public val kind: RenderLayerKind,
    public val content: String,
    public val font: String,
    public val style: RenderStyleTrace,
    @SerialName("expected_bounds") public val expectedBounds: RenderBounds,
    @SerialName("cursor_start") public val cursorStart: Int,
    @SerialName("content_cursor_start") public val contentCursorStart: Int,
    @SerialName("content_cursor_end") public val contentCursorEnd: Int,
    @SerialName("cursor_end") public val cursorEnd: Int,
    public val advance: Int,
    @SerialName("visual_width") public val visualWidth: Int,
    @SerialName("net_cursor_delta") public val netCursorDelta: Int,
    @SerialName("sprite_id") public val spriteId: String? = null,
    public val glyph: String? = null,
) {
    internal fun validate() {
        require(semanticId.length in 1..512) { "Invalid semantic layer id" }
        require(content.length <= 65_536) { "Layer content is too large" }
        require(font.length in 1..256) { "Invalid layer font" }
        expectedBounds.validate()
        require(visualWidth >= 0) { "Visual width must be non-negative" }
        require(netCursorDelta == cursorEnd - cursorStart) { "Invalid net cursor delta" }
        require(contentCursorEnd - contentCursorStart == advance) { "Invalid content advance" }
        require(spriteId == null || spriteId.length in 1..256) { "Invalid sprite id" }
        require(glyph == null || glyph.codePointCount(0, glyph.length) == 1) {
            "Sprite glyph must be one codepoint"
        }
    }
}

@Serializable
public data class RenderBounds(
    public val x: Int,
    public val y: Int,
    public val width: Int,
    public val height: Int,
) {
    internal fun validate() {
        require(width >= 0 && height >= 0) { "Render bounds dimensions must be non-negative" }
    }
}

@Serializable
public data class RenderStyleTrace(
    public val color: String,
    public val shadow: Boolean,
    public val bold: Boolean = false,
    public val italic: Boolean = false,
    public val underlined: Boolean = false,
    public val strikethrough: Boolean = false,
    public val obfuscated: Boolean = false,
)

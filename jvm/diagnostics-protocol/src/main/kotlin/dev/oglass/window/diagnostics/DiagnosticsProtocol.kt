package dev.oglass.window.diagnostics

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

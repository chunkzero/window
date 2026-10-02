package dev.oglass.window.diagnostics.minestom

import dev.oglass.window.diagnostics.CapabilityAck
import dev.oglass.window.diagnostics.DiagnosticsProtocol
import dev.oglass.window.diagnostics.RenderFrame
import org.slf4j.LoggerFactory
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

internal class DiagnosticsNegotiation(
    internal val serverMaxPayloadBytes: Int,
) {
    private val clients = ConcurrentHashMap<UUID, NegotiatedClient>()

    init {
        require(serverMaxPayloadBytes in 1024..DiagnosticsProtocol.MAX_PAYLOAD_BYTES) {
            "Minestom diagnostics payload limit must be between 1024 and " +
                DiagnosticsProtocol.MAX_PAYLOAD_BYTES
        }
    }

    fun accept(
        playerId: UUID,
        bytes: ByteArray,
    ): CapabilityAck {
        val hello = DiagnosticsProtocol.decodeHello(bytes, serverMaxPayloadBytes)
        val supported =
            DiagnosticsProtocol.SCHEMA_VERSION in
                hello.minimumSchemaVersion..hello.maximumSchemaVersion &&
                DiagnosticsProtocol.RENDER_FRAME_FEATURE in hello.features
        if (!supported) {
            clients.remove(playerId)
            return CapabilityAck(
                accepted = false,
                clientNonce = hello.clientNonce,
                reason = "No mutually supported Window diagnostics capability",
            )
        }

        val negotiatedLimit = minOf(serverMaxPayloadBytes, hello.maxPayloadBytes)
        val client = NegotiatedClient(UUID.randomUUID().toString(), negotiatedLimit)
        clients[playerId] = client
        return CapabilityAck(
            accepted = true,
            clientNonce = hello.clientNonce,
            sessionId = client.sessionId,
            maxPayloadBytes = client.maxPayloadBytes,
            features = listOf(DiagnosticsProtocol.RENDER_FRAME_FEATURE),
        )
    }

    fun encodeFrame(
        playerId: UUID,
        frame: RenderFrame,
    ): ByteArray? {
        val client = clients[playerId] ?: return null
        return try {
            DiagnosticsProtocol.encodeFrame(frame, client.maxPayloadBytes)
        } catch (error: IllegalArgumentException) {
            LOGGER.warn(
                "Dropped Window diagnostics frame {} larger than negotiated limit {}",
                frame.frameId,
                client.maxPayloadBytes,
            )
            null
        }
    }

    fun remove(playerId: UUID) {
        clients.remove(playerId)
    }

    fun clear() {
        clients.clear()
    }

    private data class NegotiatedClient(
        val sessionId: String,
        val maxPayloadBytes: Int,
    )

    private companion object {
        val LOGGER = LoggerFactory.getLogger(DiagnosticsNegotiation::class.java)
    }
}

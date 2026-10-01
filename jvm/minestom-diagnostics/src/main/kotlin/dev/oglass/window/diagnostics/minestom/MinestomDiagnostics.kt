package dev.oglass.window.diagnostics.minestom

import dev.oglass.window.RenderDiagnosticsObserver
import dev.oglass.window.diagnostics.CapabilityAck
import dev.oglass.window.diagnostics.DiagnosticsProtocol
import dev.oglass.window.diagnostics.PackFingerprint
import dev.oglass.window.diagnostics.RenderFrame
import net.minestom.server.MinecraftServer
import net.minestom.server.entity.Player
import net.minestom.server.event.Event
import net.minestom.server.event.EventNode
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerPluginMessageEvent
import net.minestom.server.network.packet.server.common.PluginMessagePacket
import org.slf4j.LoggerFactory
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

/**
 * Optional Minestom transport for Window render diagnostics.
 *
 * Installation only registers a custom-payload listener. A player receives no acknowledgement or
 * render data until its mod explicitly sends a valid v1 capability hello. Malformed, oversized, and
 * unnegotiated traffic is ignored; encoded frames are bounded by both peers' advertised limits.
 */
public class MinestomDiagnostics
    private constructor(
        private val eventRoot: EventNode<Event>,
        private val packFingerprint: PackFingerprint?,
        maxPayloadBytes: Int,
    ) : RenderDiagnosticsObserver,
        AutoCloseable {
        private val negotiation = DiagnosticsNegotiation(maxPayloadBytes)
        private val fabricNegotiation = FabricChannelNegotiation()
        private val eventNode = EventNode.all("window-diagnostics-${NODE_ID.getAndIncrement()}")

        init {
            eventNode.addListener(AsyncPlayerConfigurationEvent::class.java, ::beginFabricNegotiation)
            eventNode.addListener(PlayerPluginMessageEvent::class.java, ::handlePluginMessage)
            eventNode.addListener(PlayerDisconnectEvent::class.java) { event ->
                negotiation.remove(event.player.uuid)
                fabricNegotiation.remove(event.player.uuid)
            }
            eventRoot.addChild(eventNode)
        }

        override fun observe(
            player: Player,
            frame: RenderFrame,
        ) {
            val payload =
                negotiation.encodeFrame(
                    player.uuid,
                    frame.copy(packFingerprint = packFingerprint ?: frame.packFingerprint),
                ) ?: return
            player.sendPacket(PluginMessagePacket(DiagnosticsProtocol.FRAME_CHANNEL, payload))
        }

        private fun handlePluginMessage(event: PlayerPluginMessageEvent) {
            if (event.identifier == FabricChannelNegotiation.VERSION_CHANNEL) {
                fabricNegotiation.acceptVersion(event.player.uuid, event.message)
                return
            }
            if (event.identifier != DiagnosticsProtocol.CAPABILITY_CHANNEL) return
            val ack =
                try {
                    negotiation.accept(event.player.uuid, event.message)
                } catch (error: IllegalArgumentException) {
                    LOGGER.debug("Rejected invalid Window diagnostics capability payload", error)
                    return
                } catch (error: kotlinx.serialization.SerializationException) {
                    LOGGER.debug("Rejected malformed Window diagnostics capability payload", error)
                    return
                }
            val payload = DiagnosticsProtocol.encodeAck(ack, negotiation.serverMaxPayloadBytes)
            event.player.sendPacket(
                PluginMessagePacket(DiagnosticsProtocol.CAPABILITY_CHANNEL, payload),
            )
        }

        private fun beginFabricNegotiation(event: AsyncPlayerConfigurationEvent) {
            val player = event.player
            negotiation.remove(player.uuid)
            val version = fabricNegotiation.begin(player.uuid)
            player.sendPacket(PluginMessagePacket(FabricChannelNegotiation.VERSION_CHANNEL, version))
            // Both packets are ordered on the configuration connection. The client negotiates the
            // common version while handling c:version, before it handles this registration. Waiting
            // to send c:register from the asynchronous plugin-message event races FinishConfiguration.
            player.sendPacket(
                PluginMessagePacket(
                    FabricChannelNegotiation.REGISTER_CHANNEL,
                    FabricChannelNegotiation.REGISTER_PAYLOAD,
                ),
            )

            // A cookie request is a vanilla configuration protocol round trip. Because it is sent
            // after c:version, its response is a barrier proving that the client has already handled
            // the version payload (or ignored it, for vanilla). This avoids an arbitrary login delay
            // and keeps Minestom from sending FinishConfiguration before a Fabric client can reply.
            try {
                player.playerConnection
                    .fetchCookie(FABRIC_NEGOTIATION_BARRIER)
                    .get(FABRIC_NEGOTIATION_TIMEOUT_SECONDS, TimeUnit.SECONDS)
            } catch (error: TimeoutException) {
                LOGGER.debug("Timed out waiting for Window Fabric negotiation barrier", error)
            } catch (error: java.util.concurrent.ExecutionException) {
                LOGGER.debug("Window Fabric negotiation barrier failed", error)
            } catch (error: InterruptedException) {
                Thread.currentThread().interrupt()
                LOGGER.debug("Window Fabric negotiation barrier was interrupted", error)
            }
        }

        /** Removes this adapter's listeners and negotiated client state. */
        override fun close() {
            eventRoot.removeChild(eventNode)
            negotiation.clear()
            fabricNegotiation.clear()
        }

        public companion object {
            private val LOGGER = LoggerFactory.getLogger(MinestomDiagnostics::class.java)
            private val NODE_ID =
                java.util.concurrent.atomic
                    .AtomicLong()
            private const val FABRIC_NEGOTIATION_BARRIER = "window:diagnostics_barrier"
            private const val FABRIC_NEGOTIATION_TIMEOUT_SECONDS = 2L

            /** Installs diagnostics on Minestom's global event graph. */
            @JvmStatic
            @JvmOverloads
            public fun install(
                packFingerprint: PackFingerprint? = null,
                maxPayloadBytes: Int = DiagnosticsProtocol.MAX_PAYLOAD_BYTES,
            ): MinestomDiagnostics =
                MinestomDiagnostics(
                    MinecraftServer.getGlobalEventHandler(),
                    packFingerprint,
                    maxPayloadBytes,
                )

            /** Installs diagnostics under an explicit event root. */
            @JvmStatic
            @JvmOverloads
            public fun install(
                eventRoot: EventNode<Event>,
                packFingerprint: PackFingerprint? = null,
                maxPayloadBytes: Int = DiagnosticsProtocol.MAX_PAYLOAD_BYTES,
            ): MinestomDiagnostics = MinestomDiagnostics(eventRoot, packFingerprint, maxPayloadBytes)
        }
    }

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

package com.chunkzero.window.diagnostics.minestom

import com.chunkzero.window.diagnostics.DiagnosticsProtocol
import java.io.ByteArrayOutputStream
import java.nio.charset.StandardCharsets
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap

/** Minimal implementation of Fabric's versioned common channel-registration wire protocol. */
internal class FabricChannelNegotiation {
    private val clients = ConcurrentHashMap<UUID, State>()

    fun begin(playerId: UUID): ByteArray {
        clients[playerId] = State.VERSION_SENT
        return VERSION_PAYLOAD.clone()
    }

    fun acceptVersion(
        playerId: UUID,
        payload: ByteArray,
    ): ByteArray? {
        if (clients[playerId] != State.VERSION_SENT) return null
        val versions = decodeVarIntArray(payload) ?: return null
        if (PROTOCOL_VERSION !in versions) {
            clients[playerId] = State.UNSUPPORTED
            return null
        }
        clients[playerId] = State.REGISTERED
        return REGISTER_PAYLOAD.clone()
    }

    fun remove(playerId: UUID) {
        clients.remove(playerId)
    }

    fun clear() {
        clients.clear()
    }

    private enum class State {
        VERSION_SENT,
        REGISTERED,
        UNSUPPORTED,
    }

    internal companion object {
        const val VERSION_CHANNEL = "c:version"
        const val REGISTER_CHANNEL = "c:register"
        const val PROTOCOL_VERSION = 1

        val VERSION_PAYLOAD: ByteArray = encodeVarIntArray(intArrayOf(PROTOCOL_VERSION))
        val REGISTER_PAYLOAD: ByteArray =
            encodeRegister(
                protocol = "play",
                // ServerPlayNetworking.getGlobalReceivers() is the canonical source for this
                // direction. Frames are clientbound and are advertised by the client's reply.
                channels = listOf(DiagnosticsProtocol.CAPABILITY_CHANNEL),
            )

        fun encodeRegister(
            protocol: String,
            channels: List<String>,
        ): ByteArray =
            ByteArrayOutputStream()
                .apply {
                    writeVarInt(PROTOCOL_VERSION)
                    writeUtf(protocol)
                    writeVarInt(channels.size)
                    channels.forEach { channel -> writeUtf(channel) }
                }.toByteArray()

        fun decodeVarIntArray(payload: ByteArray): IntArray? {
            if (payload.size > MAX_VERSION_PAYLOAD_BYTES) return null
            val cursor = Cursor(payload)
            val size = cursor.readVarInt() ?: return null
            if (size !in 0..MAX_VERSIONS) return null
            val versions = IntArray(size)
            for (index in versions.indices) versions[index] = cursor.readVarInt() ?: return null
            return versions.takeIf { cursor.atEnd() }
        }

        private fun encodeVarIntArray(values: IntArray): ByteArray =
            ByteArrayOutputStream()
                .apply {
                    writeVarInt(values.size)
                    values.forEach { version -> writeVarInt(version) }
                }.toByteArray()

        private fun ByteArrayOutputStream.writeUtf(value: String) {
            val bytes = value.toByteArray(StandardCharsets.UTF_8)
            require(bytes.size <= MAX_STRING_BYTES) { "Fabric registration string is too long" }
            writeVarInt(bytes.size)
            write(bytes)
        }

        private fun ByteArrayOutputStream.writeVarInt(value: Int) {
            var remaining = value
            do {
                var byte = remaining and 0x7f
                remaining = remaining ushr 7
                if (remaining != 0) byte = byte or 0x80
                write(byte)
            } while (remaining != 0)
        }

        private const val MAX_VERSIONS = 16
        private const val MAX_VERSION_PAYLOAD_BYTES = 80
        private const val MAX_STRING_BYTES = 32_767
    }
}

private class Cursor(
    private val bytes: ByteArray,
) {
    private var index = 0

    fun readVarInt(): Int? {
        var value = 0
        for (position in 0 until 5) {
            if (index >= bytes.size) return null
            val byte = bytes[index++].toInt() and 0xff
            value = value or ((byte and 0x7f) shl (position * 7))
            if (byte and 0x80 == 0) return value
        }
        return null
    }

    fun atEnd(): Boolean = index == bytes.size
}

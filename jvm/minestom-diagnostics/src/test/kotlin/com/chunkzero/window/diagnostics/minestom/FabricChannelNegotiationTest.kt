package com.chunkzero.window.diagnostics.minestom

import com.chunkzero.window.diagnostics.DiagnosticsProtocol
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import java.nio.charset.StandardCharsets
import java.util.UUID

class FabricChannelNegotiationTest :
    StringSpec({
        "version payload is Fabric's varint array containing version one" {
            FabricChannelNegotiation.VERSION_PAYLOAD.toList() shouldBe
                listOf(0x01.toByte(), 0x01.toByte())
            FabricChannelNegotiation.decodeVarIntArray(
                FabricChannelNegotiation.VERSION_PAYLOAD,
            ) shouldBe intArrayOf(1)
        }

        "supported response emits deterministic play channel registration" {
            val negotiation = FabricChannelNegotiation()
            val playerId = UUID.randomUUID()
            negotiation.begin(playerId)

            val payload =
                negotiation.acceptVersion(playerId, FabricChannelNegotiation.VERSION_PAYLOAD)
            payload.shouldNotBeNull()
            decodeRegistration(payload) shouldBe
                Registration(
                    version = 1,
                    protocol = "play",
                    channels = listOf(DiagnosticsProtocol.CAPABILITY_CHANNEL),
                )
        }

        "unsolicited malformed and unsupported responses are ignored" {
            val negotiation = FabricChannelNegotiation()
            val playerId = UUID.randomUUID()
            negotiation.acceptVersion(playerId, byteArrayOf(1, 1)).shouldBeNull()

            negotiation.begin(playerId)
            negotiation.acceptVersion(playerId, byteArrayOf(1, 2)).shouldBeNull()

            val second = UUID.randomUUID()
            negotiation.begin(second)
            negotiation.acceptVersion(second, byteArrayOf(0x80.toByte())).shouldBeNull()
        }
    })

private data class Registration(
    val version: Int,
    val protocol: String,
    val channels: List<String>,
)

private fun decodeRegistration(bytes: ByteArray): Registration {
    var index = 0

    fun readVarInt(): Int {
        var value = 0
        for (position in 0 until 5) {
            val byte = bytes[index++].toInt() and 0xff
            value = value or ((byte and 0x7f) shl (position * 7))
            if (byte and 0x80 == 0) return value
        }
        error("invalid varint")
    }

    fun readUtf(): String {
        val size = readVarInt()
        return bytes.copyOfRange(index, index + size).toString(StandardCharsets.UTF_8).also {
            index += size
        }
    }

    val version = readVarInt()
    val protocol = readUtf()
    val channels = List(readVarInt()) { readUtf() }
    index shouldBe bytes.size
    return Registration(version, protocol, channels)
}

package com.chunkzero.window.diagnostics.minestom

import com.chunkzero.window.diagnostics.CapabilityHello
import com.chunkzero.window.diagnostics.DiagnosticsProtocol
import com.chunkzero.window.diagnostics.RenderBounds
import com.chunkzero.window.diagnostics.RenderCorrelation
import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrame
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderLayerTrace
import com.chunkzero.window.diagnostics.RenderStyleTrace
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import java.util.UUID

class DiagnosticsNegotiationTest :
    StringSpec({
        "vanilla and non-debug players receive no render payload" {
            val negotiation = DiagnosticsNegotiation(32_768)
            negotiation.encodeFrame(UUID.randomUUID(), frame()) shouldBe null
        }

        "valid hello negotiates the smaller payload limit and enables frames" {
            val negotiation = DiagnosticsNegotiation(32_768)
            val playerId = UUID.randomUUID()
            val hello = CapabilityHello(clientNonce = "test", maxPayloadBytes = 16_384)
            val ack = negotiation.accept(playerId, DiagnosticsProtocol.encodeHello(hello))

            ack.accepted shouldBe true
            ack.clientNonce shouldBe "test"
            ack.maxPayloadBytes shouldBe 16_384
            ack.sessionId.shouldNotBeNull()
            DiagnosticsProtocol.decodeFrame(negotiation.encodeFrame(playerId, frame())!!) shouldBe
                frame()
        }

        "unsupported feature is rejected and does not enable frames" {
            val negotiation = DiagnosticsNegotiation(32_768)
            val playerId = UUID.randomUUID()
            val hello = CapabilityHello(clientNonce = "test", features = listOf("unknown"))
            val ack = negotiation.accept(playerId, DiagnosticsProtocol.encodeHello(hello))

            ack.accepted shouldBe false
            negotiation.encodeFrame(playerId, frame()).shouldBeNull()
        }

        "a negotiated small limit safely drops oversized frames" {
            val negotiation = DiagnosticsNegotiation(32_768)
            val playerId = UUID.randomUUID()
            val hello = CapabilityHello(clientNonce = "test", maxPayloadBytes = 1024)
            negotiation.accept(playerId, DiagnosticsProtocol.encodeHello(hello))

            val oversized = frame().copy(layers = List(30) { index -> layer("layer-$index") })
            negotiation.encodeFrame(playerId, oversized).shouldBeNull()
        }
    })

private fun frame(): RenderFrame =
    RenderFrame(
        frameId = 1,
        reason = RenderFrameReason.OPEN,
        correlation =
            RenderCorrelation(RenderSurfaceKind.WINDOW, "shop", "render-session", containerId = 2),
        cursorConvention = RenderCursorConvention.INDEPENDENT_NET_ZERO_SEGMENTS,
        cursorStart = 8,
        cursorEnd = 8,
        netCursorDelta = 0,
        layers = listOf(layer("window/shop/static")),
    )

private fun layer(id: String): RenderLayerTrace =
    RenderLayerTrace(
        semanticId = id,
        kind = RenderLayerKind.STATIC_CHROME,
        content = "x".repeat(128),
        font = "window:ui",
        style = RenderStyleTrace("#ffffff", false),
        expectedBounds = RenderBounds(0, 0, 176, 222),
        cursorStart = 8,
        contentCursorStart = 8,
        contentCursorEnd = 8,
        cursorEnd = 8,
        advance = 0,
        visualWidth = 176,
        netCursorDelta = 0,
    )

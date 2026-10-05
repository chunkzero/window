package com.chunkzero.window.diagnostics

import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class DiagnosticsProtocolTest :
    StringSpec({
        "capability messages round trip with canonical defaults" {
            val hello = CapabilityHello(clientNonce = "client-1")
            DiagnosticsProtocol.decodeHello(DiagnosticsProtocol.encodeHello(hello)) shouldBe hello

            val ack =
                CapabilityAck(
                    accepted = true,
                    clientNonce = hello.clientNonce,
                    sessionId = "session-1",
                    maxPayloadBytes = 32_768,
                    features = listOf(DiagnosticsProtocol.RENDER_FRAME_FEATURE),
                )
            DiagnosticsProtocol.decodeAck(DiagnosticsProtocol.encodeAck(ack)) shouldBe ack
        }

        "render frame round trips without losing cursor correlation" {
            val frame = testFrame()
            DiagnosticsProtocol.decodeFrame(DiagnosticsProtocol.encodeFrame(frame)) shouldBe frame
        }

        "decoder rejects oversized and malformed untrusted input" {
            shouldThrow<IllegalArgumentException> {
                DiagnosticsProtocol.decodeHello(ByteArray(1025), maxBytes = 1024)
            }
            shouldThrow<kotlinx.serialization.SerializationException> {
                DiagnosticsProtocol.decodeHello(byteArrayOf(0xc3.toByte(), 0x28))
            }
        }

        "frame validation rejects a false net-zero claim" {
            val bad =
                testFrame().copy(layers = listOf(testFrame().layers.single().copy(cursorEnd = 9)))
            shouldThrow<IllegalArgumentException> {
                DiagnosticsProtocol.decodeFrame(DiagnosticsProtocol.encodeFrame(bad))
            }
        }
    })

private fun testFrame(): RenderFrame =
    RenderFrame(
        frameId = 1,
        reason = RenderFrameReason.OPEN,
        correlation =
            RenderCorrelation(
                surfaceKind = RenderSurfaceKind.WINDOW,
                semanticId = "shop",
                renderSessionId = "render-1",
                containerId = 4,
            ),
        cursorConvention = RenderCursorConvention.INDEPENDENT_NET_ZERO_SEGMENTS,
        cursorStart = 8,
        cursorEnd = 8,
        netCursorDelta = 0,
        packFingerprint = PackFingerprint("sha256", "a".repeat(64)),
        layers =
            listOf(
                RenderLayerTrace(
                    semanticId = "window/shop/title",
                    kind = RenderLayerKind.TEXT_SLOT,
                    content = "Shop",
                    font = "window:y0",
                    style = RenderStyleTrace("#404040", shadow = false),
                    expectedBounds = RenderBounds(8, 6, 19, 8),
                    cursorStart = 8,
                    contentCursorStart = 8,
                    contentCursorEnd = 28,
                    cursorEnd = 8,
                    advance = 20,
                    visualWidth = 19,
                    netCursorDelta = 0,
                ),
            ),
    )

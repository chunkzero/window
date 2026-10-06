package com.chunkzero.window

import com.chunkzero.window.diagnostics.RenderCursorConvention
import com.chunkzero.window.diagnostics.RenderFrame
import com.chunkzero.window.diagnostics.RenderFrameReason
import com.chunkzero.window.diagnostics.RenderLayerKind
import com.chunkzero.window.diagnostics.RenderSurfaceKind
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.HudShaderEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component

class RenderDiagnosticsTest :
    StringSpec({
        afterTest { WindowDiagnostics.observer = RenderDiagnosticsObserver.NONE }

        "window session observes the same open and reactive composition passes" {
            val manifest =
                TestManifests.manifest(
                    slots =
                        linkedMapOf(
                            "label" to TestManifests.slot(8, 40, Align.LEFT).copy(text = "Static"),
                            "count" to TestManifests.slot(60, 40, Align.RIGHT),
                        ),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var count by state(1)

                    override fun WindowScope<Any>.bind() {
                        slot("count") { Component.text(count.toString()) }
                    }
                }
            val frames = mutableListOf<RenderFrame>()
            val observed = mutableListOf<Any>()
            WindowDiagnostics.observer =
                RenderDiagnosticsObserver { source, frame ->
                    observed += source
                    frames += frame
                }
            val scheduler = host.scheduler
            view.open()

            view.count = 2
            scheduler.runAll()

            frames.map { it.frameId } shouldContainExactly listOf(1, 2)
            frames.map { it.reason } shouldContainExactly
                listOf(RenderFrameReason.OPEN, RenderFrameReason.REACTIVE_UPDATE)
            frames.first().correlation.surfaceKind shouldBe RenderSurfaceKind.WINDOW
            frames.first().correlation.semanticId shouldBe "w"
            observed.distinct() shouldContainExactly listOf(host)
            frames.first().correlation.renderSessionId shouldBe
                frames.last().correlation.renderSessionId
            frames.first().layers.map { it.semanticId } shouldContainExactly
                listOf("window/w/static", "window/w/slot/label", "window/w/slot/count")
            frames
                .last()
                .layers
                .single { it.semanticId.endsWith("/count") }
                .content shouldBe "2"
        }

        "title text and sprite traces make the independent net-zero convention explicit" {
            val manifest =
                TestManifests.manifest(
                    slots = mapOf("title" to TestManifests.slot(20, 50, Align.CENTER)),
                    spriteSlots =
                        mapOf(
                            "icon" to
                                TestManifests.spriteSlot(
                                    x = 8,
                                    y = 20,
                                    width = 20,
                                    height = 12,
                                    align = Align.CENTER,
                                    font = "window:y0",
                                ),
                        ),
                    sprites = mapOf("coin" to TestManifests.sprite(width = 12, advance = 13)),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("Hi") }
                        sprite("icon") { "coin" }
                    }
                }
            var frame: RenderFrame? = null
            WindowDiagnostics.observer = RenderDiagnosticsObserver { _, value -> frame = value }
            view.open()

            val layers = frame!!.layers.filter { it.kind != RenderLayerKind.STATIC_CHROME }
            layers.map { it.kind } shouldContainExactly
                listOf(RenderLayerKind.SPRITE_SLOT, RenderLayerKind.TEXT_SLOT)
            layers.first().style.shadow shouldBe false
            layers.forEach { layer ->
                layer.cursorEnd shouldBe layer.cursorStart
                layer.netCursorDelta shouldBe 0
                (layer.contentCursorEnd - layer.contentCursorStart) shouldBe layer.advance
            }
        }

        "observer failures never change normal title delivery" {
            val manifest = TestManifests.manifest()
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest, host) {
                    override fun WindowScope<Any>.bind() = Unit
                }
            WindowDiagnostics.observer = RenderDiagnosticsObserver { _, _ -> error("boom") }
            view.open()

            handle.opened shouldBe true
            handle.titles.size shouldBe 1
        }

        "non-shader HUD observer reports full fixed-width advance and actual layer deltas" {
            val manifest =
                TestManifests.hudManifest(
                    width = 100,
                    slots = mapOf("coins" to TestManifests.slot(10, 30, Align.LEFT, y = 2)),
                )
            val view =
                object : TestHud(manifest) {
                    var coins = 1

                    override fun HudScope.bind() {
                        slot("coins") { Component.text("$coins") }
                    }
                }
            val frames = mutableListOf<RenderFrame>()
            WindowDiagnostics.observer = HudObserver { hud, frame -> if (hud === view) frames += frame }
            view.render()

            view.coins = 2
            view.render()
            view.render()

            frames.map { it.reason } shouldContainExactly
                listOf(RenderFrameReason.OPEN, RenderFrameReason.REACTIVE_UPDATE)
            val frame = frames.first()
            frame.cursorConvention shouldBe RenderCursorConvention.FIXED_WIDTH_COMPOSITION
            frame.cursorStart shouldBe 0
            frame.cursorEnd shouldBe 100
            frame.netCursorDelta shouldBe 100
            val static = frame.layers.first()
            static.kind shouldBe RenderLayerKind.HUD_STATIC
            static.cursorStart shouldBe 0
            static.cursorEnd shouldBe 100
            static.advance shouldBe 100
            static.netCursorDelta shouldBe 100
            (static.contentCursorEnd - static.contentCursorStart) shouldBe 100
            val slot = frame.layers.last()
            slot.kind shouldBe RenderLayerKind.HUD_TEXT
            slot.netCursorDelta shouldBe (slot.cursorEnd - slot.cursorStart)
            (slot.netCursorDelta == 0) shouldBe false
            frames
                .last()
                .layers
                .last()
                .content shouldBe "2"
        }

        "shader HUD observer reports independently reset slot geometry inside fixed-width frame" {
            val manifest =
                TestManifests.hudManifest(
                    width = 100,
                    slots =
                        mapOf(
                            "coins" to
                                TestManifests.slot(
                                    10,
                                    30,
                                    Align.RIGHT,
                                    y = 2,
                                    shaderMarker = "#120034",
                                ),
                        ),
                    shader = HudShaderEntry(staticMarker = "#010002", sourceBottom = 59),
                )
            val view =
                object : TestHud(manifest) {
                    override fun HudScope.bind() {
                        slot("coins") { Component.text("Hi") }
                    }
                }
            var frame: RenderFrame? = null
            WindowDiagnostics.observer = HudObserver { _, value -> frame = value }
            view.render()

            val observed = frame!!
            observed.cursorConvention shouldBe RenderCursorConvention.FIXED_WIDTH_COMPOSITION
            val slot = observed.layers.last()
            slot.cursorStart shouldBe 0
            slot.cursorEnd shouldBe 0
            slot.netCursorDelta shouldBe 0
            slot.expectedBounds.x shouldBe 33
            slot.advance shouldBe 8
            slot.visualWidth shouldBe 7
        }
    })

/** An observer of HUD frames only. */
private class HudObserver(
    private val observe: (HudView, RenderFrame) -> Unit,
) : RenderDiagnosticsObserver {
    override fun observe(
        host: WindowHost<*>,
        frame: RenderFrame,
    ) = Unit

    override fun observeHud(
        hud: HudView,
        frame: RenderFrame,
    ) = observe.invoke(hud, frame)
}

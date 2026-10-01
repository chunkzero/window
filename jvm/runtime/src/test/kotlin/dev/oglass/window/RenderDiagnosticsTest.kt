package dev.oglass.window

import dev.oglass.window.diagnostics.RenderCursorConvention
import dev.oglass.window.diagnostics.RenderFrame
import dev.oglass.window.diagnostics.RenderFrameReason
import dev.oglass.window.diagnostics.RenderLayerKind
import dev.oglass.window.diagnostics.RenderSurfaceKind
import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.HudShaderEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component

class RenderDiagnosticsTest :
    StringSpec({
        "window session observes the same open and reactive composition passes" {
            val manifest =
                TestManifests.manifest(
                    slots =
                        linkedMapOf(
                            "label" to TestManifests.slot(8, 40, Align.LEFT).copy(text = "Static"),
                            "count" to TestManifests.slot(60, 40, Align.RIGHT),
                        ),
                )
            val view =
                object : WindowView("w") {
                    var count by state(1)

                    override fun WindowScope.bind() {
                        slot("count") { Component.text(count.toString()) }
                    }
                }
            val frames = mutableListOf<RenderFrame>()
            val observer = RenderDiagnosticsObserver { _, frame -> frames += frame }
            val scheduler = ManualScheduler()
            testSession(
                manifest,
                "w",
                view,
                scheduler,
                FakeInventoryHandle(),
                diagnosticsObserver = observer,
            ).open()

            view.count = 2
            scheduler.runAll()

            frames.map { it.frameId } shouldContainExactly listOf(1, 2)
            frames.map { it.reason } shouldContainExactly
                listOf(RenderFrameReason.OPEN, RenderFrameReason.REACTIVE_UPDATE)
            frames.first().correlation.surfaceKind shouldBe RenderSurfaceKind.WINDOW
            frames.first().correlation.semanticId shouldBe "w"
            frames.first().correlation.containerId shouldBe 7
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
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("Hi") }
                        sprite("icon") { "coin" }
                    }
                }
            var frame: RenderFrame? = null
            testSession(
                manifest,
                "w",
                view,
                ManualScheduler(),
                FakeInventoryHandle(),
                diagnosticsObserver = RenderDiagnosticsObserver { _, value -> frame = value },
            ).open()

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
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() = Unit
                }
            testSession(
                manifest,
                "w",
                view,
                ManualScheduler(),
                handle,
                diagnosticsObserver = RenderDiagnosticsObserver { _, _ -> error("boom") },
            ).open()

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
                object : HudView("h") {
                    var coins by state(1)

                    override fun HudScope.bind() {
                        slot("coins") { Component.text("$coins") }
                    }
                }
            val frames = mutableListOf<RenderFrame>()
            val sent = mutableListOf<Component>()
            val scheduler = ManualScheduler()
            HudSession(
                HudDefinition("h", manifest, manifest.huds.getValue("h")),
                view,
                stubPlayer,
                scheduler,
                RenderDiagnosticsObserver { _, frame -> frames += frame },
                sent::add,
            ).show()

            view.coins = 2
            scheduler.runAll()

            sent.size shouldBe 2
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
                object : HudView("h") {
                    override fun HudScope.bind() {
                        slot("coins") { Component.text("Hi") }
                    }
                }
            var frame: RenderFrame? = null
            HudSession(
                HudDefinition("h", manifest, manifest.huds.getValue("h")),
                view,
                stubPlayer,
                ManualScheduler(),
                RenderDiagnosticsObserver { _, value -> frame = value },
                {},
            ).show()

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

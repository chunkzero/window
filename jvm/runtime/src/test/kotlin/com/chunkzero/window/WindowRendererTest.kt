package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.internal.ComposedRender
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.RenderKey
import com.chunkzero.window.internal.SlotWrites
import com.chunkzero.window.internal.Switches
import com.chunkzero.window.internal.WindowBindings
import com.chunkzero.window.internal.WindowFrame
import com.chunkzero.window.internal.WindowRenderer
import com.chunkzero.window.internal.titleSlots
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ButtonState
import com.chunkzero.window.manifest.ItemEntry
import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry
import com.chunkzero.window.manifest.TooltipEntry
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import io.kotest.matchers.string.shouldContain
import io.kotest.matchers.string.shouldNotContain
import io.kotest.matchers.types.shouldBeSameInstanceAs
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

/** A renderer over [manifest]'s window `w` whose items are [WindowItem]s; dirty keys re-render on [scheduler] ticks. */
private class RendererFixture(
    manifest: WindowManifest,
) {
    val scheduler = ManualScheduler()
    val frames = mutableListOf<WindowFrame<Any>>()
    val reactivity: Reactivity = Reactivity(scheduler) { frames += renderer.render(it) }
    private val definition = WindowDefinition(manifest, "w")
    private val bindings =
        WindowBindings<Any>(
            definition,
            { it },
            definition.titleSlots(reactivity),
            Switches("window", "w", definition.entry.switches, reactivity),
        )
    val renderer = WindowRenderer(definition, bindings, reactivity) { it }

    fun open(bind: WindowScope<Any>.() -> Unit): ComposedRender {
        bindings.bind()
        bindings.validate()
        return renderer.seedTitle()
    }
}

private fun plain(render: ComposedRender): String = PlainTextComponentSerializer.plainText().serialize(render.component)

private fun SlotWrites<Any>.model(slot: Int): String =
    (items.getValue(SlotRef(SlotArea.CONTAINER, slot)) as WindowItem.Hitbox).model.asString()

class WindowRendererTest :
    StringSpec({
        "title draws only the active switch case's art and slots" {
            val manifest =
                TestManifests.manifest(
                    slots =
                        mapOf(
                            "price" to TestManifests.slot(8, 40, Align.LEFT),
                            "sell_label" to TestManifests.slot(8, 40, Align.LEFT, text = "Sell"),
                        ),
                    switches =
                        mapOf(
                            "mode" to
                                SwitchEntry(
                                    listOf(
                                        SwitchCaseEntry("buy", static = "BUYART", slots = listOf("price")),
                                        SwitchCaseEntry("sell", static = "SELLART", slots = listOf("sell_label")),
                                    ),
                                ),
                        ),
                )
            val fixture = RendererFixture(manifest)
            var mode by fixture.reactivity.state("buy")

            val opened =
                plain(
                    fixture.open {
                        slot("price") { Component.text("Ten") }
                        switch("mode") { mode }
                    },
                )
            opened shouldContain "BUYART"
            opened shouldContain "Ten"
            opened shouldNotContain "SELLART"
            opened shouldNotContain "Sell"

            mode = "sell"
            fixture.scheduler.runAll()
            val swapped = plain(fixture.frames.single().title)
            swapped shouldContain "SELLART"
            swapped shouldContain "Sell"
            swapped shouldNotContain "BUYART"
            swapped shouldNotContain "Ten"
        }

        "a toggle state change re-renders the covered button item in one scheduled frame" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    buttons =
                        mapOf(
                            "mode" to
                                TestManifests.button(
                                    slots = listOf(0),
                                    states =
                                        mapOf(
                                            "on" to
                                                ButtonState(
                                                    itemModel = "demo:gui/on",
                                                    tooltip = TooltipEntry("Enabled"),
                                                ),
                                            "off" to
                                                ButtonState(
                                                    itemModel = "demo:gui/off",
                                                    tooltip = TooltipEntry("Disabled"),
                                                ),
                                        ),
                                ),
                        ),
                )
            val fixture = RendererFixture(manifest)
            var enabled by fixture.reactivity.state(false)
            fixture.open { toggle("mode", selected = { enabled }) {} }

            fixture.renderer.seedItems().model(0) shouldBe "demo:gui/off"
            enabled = true
            fixture.scheduler.scheduleCount shouldBe 1
            fixture.scheduler.runAll()
            fixture.frames
                .single()
                .writes
                .model(0) shouldBe "demo:gui/on"
        }

        "button state sprites draw beneath fixed icons and labels, including states set after open" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    slots =
                        mapOf(
                            "label" to TestManifests.slot(x = 8, width = 40, align = Align.CENTER, text = "Mode"),
                        ),
                    spriteSlots =
                        mapOf(
                            "icon" to TestManifests.spriteSlot(x = 10, y = 6, width = 8, height = 8, sprite = "icon"),
                        ),
                    buttons =
                        mapOf(
                            "mode" to
                                TestManifests.button(
                                    slots = listOf(0),
                                    width = 40,
                                    states =
                                        mapOf(
                                            "on" to ButtonState(sprite = "selected"),
                                            "off" to ButtonState(sprite = "normal"),
                                        ),
                                    spriteFont = "window:sprite_y0",
                                ),
                        ),
                    sprites =
                        mapOf(
                            "normal" to TestManifests.sprite(width = 40, glyph = ""),
                            "selected" to TestManifests.sprite(width = 40, glyph = ""),
                            "icon" to TestManifests.sprite(width = 8, height = 8, glyph = ""),
                        ),
                )
            val fixture = RendererFixture(manifest)
            val initial = plain(fixture.open { button("mode") {} })
            initial shouldNotContain ""
            (initial.indexOf('') in 0..<initial.indexOf("Mode")) shouldBe true

            val off =
                plain(
                    fixture.renderer
                        .setButtonState("mode", "off")
                        .shouldNotBeNull()
                        .title,
                )
            (off.indexOf('') in 0..<off.indexOf('')) shouldBe true
            fixture.renderer.setButtonState("mode", "off").shouldBeNull()

            val on =
                plain(
                    fixture.renderer
                        .setButtonState("mode", "on")
                        .shouldNotBeNull()
                        .title,
                )
            (on.indexOf('') in 0..<on.indexOf('')) shouldBe true
            on shouldNotContain ""
        }

        "slot writes rendered before a provider throws stay drainable" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    buttons =
                        mapOf(
                            "mode" to
                                TestManifests.button(
                                    slots = listOf(0),
                                    states =
                                        mapOf(
                                            "on" to ButtonState(itemModel = "demo:gui/on"),
                                            "off" to ButtonState(itemModel = "demo:gui/off"),
                                        ),
                                ),
                        ),
                    items = mapOf("extra" to ItemEntry(listOf(TestManifests.containerSlot(1)))),
                )
            val fixture = RendererFixture(manifest)
            fixture.open {
                button("mode") {}
                buttonState("mode", "on")
                item("extra") { error("boom") }
            }

            shouldThrow<IllegalStateException> {
                fixture.renderer.render(setOf(RenderKey.ButtonState("mode"), RenderKey.Item("extra")))
            }
            fixture.renderer.drainWrites().model(0) shouldBe "demo:gui/on"
        }

        "inventory-only renders reuse the composed title until a title segment changes" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    slots = mapOf("label" to TestManifests.slot(8, 40, Align.LEFT)),
                    items = mapOf("extra" to ItemEntry(listOf(TestManifests.containerSlot(1)))),
                )
            val fixture = RendererFixture(manifest)
            var label by fixture.reactivity.state("A")
            val opened =
                fixture.open {
                    slot("label") { Component.text(label) }
                    item("extra") { null }
                }

            fixture.renderer.render(setOf(RenderKey.Item("extra"))).title shouldBeSameInstanceAs opened

            label = "B"
            val relabeled = fixture.renderer.render(setOf(RenderKey.Slot("label"))).title
            relabeled shouldNotBe opened
            plain(relabeled) shouldContain "B"
            fixture.renderer.render(setOf(RenderKey.Item("extra"))).title shouldBeSameInstanceAs relabeled
        }
    })

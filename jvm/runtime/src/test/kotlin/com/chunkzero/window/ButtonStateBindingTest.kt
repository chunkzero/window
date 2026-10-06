package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.ButtonState
import com.chunkzero.window.manifest.TooltipEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class ButtonStateBindingTest :
    StringSpec({
        fun FakeContainer.model(slot: Int) =
            when (val item = items.getValue(SlotRef(SlotArea.CONTAINER, slot))) {
                is WindowItem.Hitbox -> item.model.asString()
                is WindowItem.AnvilSeed -> item.model.asString()
                else -> error("Unexpected item $item")
            }

        "toggle state re-renders the covered button item" {
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
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var enabled by state(false)

                    override fun WindowScope<Any>.bind() {
                        toggle("mode", selected = { enabled }) {}
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.model(0) shouldBe "demo:gui/off"
            view.enabled = true
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
            handle.model(0) shouldBe "demo:gui/on"
        }

        "toggle state updates title sprite and keeps fixed icons below labels" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    slots =
                        mapOf(
                            "label" to
                                TestManifests.slot(
                                    x = 8,
                                    width = 40,
                                    align = Align.CENTER,
                                    text = "Mode",
                                ),
                        ),
                    spriteSlots =
                        mapOf(
                            "icon" to
                                TestManifests.spriteSlot(
                                    x = 10,
                                    y = 6,
                                    width = 8,
                                    height = 8,
                                    sprite = "icon",
                                ),
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
                            "normal" to TestManifests.sprite(width = 40, glyph = "\uE100"),
                            "selected" to TestManifests.sprite(width = 40, glyph = "\uE101"),
                            "icon" to TestManifests.sprite(width = 8, height = 8, glyph = "\uE102"),
                        ),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var enabled by state(false)

                    override fun WindowScope<Any>.bind() {
                        toggle("mode", selected = { enabled }) {}
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            val plain = PlainTextComponentSerializer.plainText()

            val initial = plain.serialize(handle.titles.last())
            (initial.indexOf('\uE100') < initial.indexOf('\uE102')) shouldBe true
            (initial.indexOf('\uE102') < initial.indexOf("Mode")) shouldBe true

            view.enabled = true
            scheduler.runAll()
            val selected = plain.serialize(handle.titles.last())
            selected.contains('\uE101') shouldBe true
            selected.contains('\uE100') shouldBe false
        }

        "state sprites set after open still draw beneath labels" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    slots =
                        mapOf(
                            "label" to TestManifests.slot(x = 8, width = 40, align = Align.CENTER, text = "Mode"),
                        ),
                    buttons =
                        mapOf(
                            "mode" to
                                TestManifests.button(
                                    slots = listOf(0),
                                    width = 40,
                                    states = mapOf("on" to ButtonState(sprite = "selected")),
                                    spriteFont = "window:sprite_y0",
                                ),
                        ),
                    sprites = mapOf("selected" to TestManifests.sprite(width = 40, glyph = "\uE101")),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    override fun WindowScope<Any>.bind() {
                        button("mode") {}
                    }

                    override fun onOpen() = buttonState("mode", "on")
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
            scheduler.runAll()

            val title = PlainTextComponentSerializer.plainText().serialize(handle.titles.last())
            (title.indexOf('\uE101') in 0..<title.indexOf("Mode")) shouldBe true
        }

        "choice buttons are mutually exclusive and route their typed value" {
            val states =
                mapOf(
                    "selected" to ButtonState(itemModel = "demo:gui/selected"),
                    "unselected" to ButtonState(itemModel = "demo:gui/unselected"),
                )
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    buttons =
                        mapOf(
                            "best" to TestManifests.button(listOf(0), states = states),
                            "new" to TestManifests.button(listOf(1), states = states),
                        ),
                )
            val selectedValues = mutableListOf<String>()
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var selected by state("best")

                    override fun WindowScope<Any>.bind() {
                        choice("best", "best", { selected }) { value, _ ->
                            selectedValues += value
                            selected = value
                        }
                        choice("new", "new", { selected }) { value, _ ->
                            selectedValues += value
                            selected = value
                        }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.model(0) shouldBe "demo:gui/selected"
            handle.model(1) shouldBe "demo:gui/unselected"
            handle.clickContainer(1)
            scheduler.runAll()
            selectedValues shouldBe listOf("new")
            handle.model(0) shouldBe "demo:gui/unselected"
            handle.model(1) shouldBe "demo:gui/selected"
        }

        "disabled buttons suppress clicks and anvil input reaches its binding" {
            val states =
                mapOf(
                    "enabled" to ButtonState(itemModel = "demo:gui/enabled"),
                    "disabled" to ButtonState(itemModel = "demo:gui/disabled"),
                )
            val manifest =
                TestManifests.manifest(
                    container = "anvil",
                    titleOrigin = listOf(60, 6),
                    buttons = mapOf("confirm" to TestManifests.button(listOf(2), states = states)),
                    inputs =
                        mapOf(
                            "query" to
                                AnvilInputEntry(
                                    slot = TestManifests.containerSlot(0),
                                    initial = "Search...",
                                    itemModel = "demo:gui/search",
                                ),
                        ),
                )
            val queries = mutableListOf<String>()
            var confirmations = 0
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var enabled by state(false)

                    override fun WindowScope<Any>.bind() {
                        anvilInput("query") { queries += it }
                        enabledButton("confirm", { enabled }) { confirmations++ }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.model(0) shouldBe "demo:gui/search"
            handle.input("maps")
            queries shouldBe listOf("maps")
            handle.clickContainer(2)
            confirmations shouldBe 0
            view.enabled = true
            scheduler.runAll()
            handle.clickContainer(2)
            confirmations shouldBe 1
        }
    })

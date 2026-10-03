package dev.oglass.window

import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.AnvilInputEntry
import dev.oglass.window.manifest.ButtonState
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import net.minestom.server.component.DataComponents
import dev.oglass.window.manifest.ButtonTooltip as ManifestTooltip

class ButtonStateBindingTest :
    StringSpec({
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
                                                    tooltip = ManifestTooltip("Enabled"),
                                                ),
                                            "off" to
                                                ButtonState(
                                                    itemModel = "demo:gui/off",
                                                    tooltip = ManifestTooltip("Disabled"),
                                                ),
                                        ),
                                ),
                        ),
                )
            val view =
                object : WindowView("w") {
                    var enabled by state(false)

                    override fun WindowScope.bind() {
                        toggle("mode", selected = { enabled }) {}
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()

            val slot = SlotRef(SlotArea.CONTAINER, 0)
            handle.items.getValue(slot).get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/off"
            view.enabled = true
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
            handle.items.getValue(slot).get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/on"
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
            val view =
                object : WindowView("w") {
                    var enabled by state(false)

                    override fun WindowScope.bind() {
                        toggle("mode", selected = { enabled }) {}
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
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
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("mode") {}
                    }

                    override fun onOpen() = buttonState("mode", "on")
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
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
            val view =
                object : WindowView("w") {
                    var selected by state("best")

                    override fun WindowScope.bind() {
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
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()

            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 0))
                .get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/selected"
            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 1))
                .get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/unselected"
            handle.clickContainer(1)
            scheduler.runAll()
            selectedValues shouldBe listOf("new")
            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 0))
                .get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/unselected"
            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 1))
                .get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/selected"
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
            val view =
                object : WindowView("w") {
                    var enabled by state(false)

                    override fun WindowScope.bind() {
                        anvilInput("query") { queries += it }
                        enabledButton("confirm", { enabled }) { confirmations++ }
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()

            val input = handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0))
            input.get(DataComponents.ITEM_MODEL) shouldBe "demo:gui/search"
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

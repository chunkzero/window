package dev.oglass.window

import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.AnvilInputEntry
import dev.oglass.window.manifest.ButtonState
import dev.oglass.window.manifest.CollectionEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import net.minestom.server.component.DataComponents
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import dev.oglass.window.manifest.ButtonTooltip as ManifestTooltip

class StateTrackingTest :
    StringSpec({
        fun twoSlotManifest() =
            TestManifests.manifest(
                container = "generic_9x3",
                slots =
                    mapOf(
                        "a" to TestManifests.slot(x = 8, width = 50, align = Align.LEFT),
                        "b" to TestManifests.slot(x = 8, width = 50, align = Align.LEFT),
                    ),
            )

        "mutating a state re-evaluates only the dependent slot, one schedule per burst" {
            var aRenders = 0
            var bRenders = 0

            val view =
                object : WindowView("w") {
                    var countA by state(0)
                    var countB by state(0)

                    override fun WindowScope.bind() {
                        slot("a") {
                            aRenders++
                            Component.text("a$countA")
                        }
                        slot("b") {
                            bRenders++
                            Component.text("b$countB")
                        }
                    }
                }

            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            val session = testSession(twoSlotManifest(), "w", view, scheduler, handle)
            session.open()

            // Open seeds both slots once.
            aRenders shouldBe 1
            bRenders shouldBe 1
            scheduler.scheduleCount shouldBe 0

            // Mutate state read by slot "a" only.
            view.countA = 1
            // A single flush is scheduled.
            scheduler.scheduleCount shouldBe 1
            scheduler.pending shouldBe 1

            scheduler.runAll()
            // Only slot "a" re-rendered.
            aRenders shouldBe 2
            bRenders shouldBe 1
            // A title update was sent.
            handle.titles.size shouldBe 2 // open + one re-render
        }

        "multiple writes in one burst coalesce into a single schedule" {
            val view =
                object : WindowView("w") {
                    var c by state(0)

                    override fun WindowScope.bind() {
                        slot("a") { Component.text("a$c") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = ManualScheduler()
            val session =
                testSession(twoSlotManifest(), "w", view, scheduler, FakeInventoryHandle())
            session.open()

            view.c = 1
            view.c = 2
            view.c = 3
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
        }

        "a state never read by any slot triggers no re-render" {
            val view =
                object : WindowView("w") {
                    var unused by state(0)

                    override fun WindowScope.bind() {
                        slot("a") { Component.text("a") }
                        slot("b") { Component.text("b") }
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(twoSlotManifest(), "w", view, scheduler, handle).open()

            view.unused = 99
            scheduler.scheduleCount shouldBe 0
            handle.titles.size shouldBe 1 // only the open
        }

        "refresh marks all slots dirty with a single schedule" {
            var aRenders = 0
            var bRenders = 0
            val view =
                object : WindowView("w") {
                    fun forceRefresh() = refresh()

                    override fun WindowScope.bind() {
                        slot("a") {
                            aRenders++
                            Component.text("a")
                        }
                        slot("b") {
                            bRenders++
                            Component.text("b")
                        }
                    }
                }
            val scheduler = ManualScheduler()
            testSession(twoSlotManifest(), "w", view, scheduler, FakeInventoryHandle()).open()
            aRenders shouldBe 1
            bRenders shouldBe 1

            view.forceRefresh()
            scheduler.scheduleCount shouldBe 1
            scheduler.runAll()
            aRenders shouldBe 2
            bRenders shouldBe 2
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
            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 0))
                .get(DataComponents.CUSTOM_NAME) shouldBe Component.text("maps")
            handle.clickContainer(2)
            confirmations shouldBe 0
            view.enabled = true
            scheduler.runAll()
            handle.clickContainer(2)
            confirmations shouldBe 1
        }

        "collection cells can render and route clicks from player inventory slots" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    collections =
                        mapOf(
                            "pets" to
                                CollectionEntry(
                                    slots =
                                        listOf(
                                            TestManifests.containerSlot(0),
                                            TestManifests.playerSlot(9),
                                        ),
                                    action = true,
                                ),
                        ),
                )
            val clicks = mutableListOf<IndexedClick>()
            val view =
                object : WindowView("w") {
                    var selected by state(0)

                    override fun WindowScope.bind() {
                        collection(
                            "pets",
                            render = { index ->
                                val material =
                                    if (index == selected) Material.DIAMOND else Material.PAPER
                                ItemStack.of(material)
                            },
                            handler = { clicks += it },
                        )
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()

            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)).material() shouldBe
                Material.DIAMOND
            handle.items.getValue(SlotRef(SlotArea.PLAYER, 9)).material() shouldBe Material.PAPER

            handle.click(SlotRef(SlotArea.PLAYER, 9), right = true)
            clicks.single().index shouldBe 1
            clicks.single().slot shouldBe SlotRef(SlotArea.PLAYER, 9)
            clicks.single().right shouldBe true

            view.selected = 1
            scheduler.runAll()
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)).material() shouldBe Material.PAPER
            handle.items.getValue(SlotRef(SlotArea.PLAYER, 9)).material() shouldBe Material.DIAMOND
        }
    })

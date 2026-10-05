package com.chunkzero.window

import com.chunkzero.window.manifest.CollectionEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material

class CollectionRoutingTest :
    StringSpec({
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

        "collection selection draws its sprite over the selected cell only" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    collections =
                        mapOf(
                            "pets" to
                                CollectionEntry(
                                    slots = listOf(TestManifests.containerSlot(0), TestManifests.containerSlot(1)),
                                    action = false,
                                    selection =
                                        listOf(
                                            TestManifests.spriteSlot(
                                                x = 7,
                                                y = 17,
                                                width = 18,
                                                height = 18,
                                                sprite = "box",
                                            ),
                                            TestManifests.spriteSlot(
                                                x = 25,
                                                y = 17,
                                                width = 18,
                                                height = 18,
                                                sprite = "box",
                                            ),
                                        ),
                                ),
                        ),
                    sprites = mapOf("box" to TestManifests.sprite(width = 18, height = 18, glyph = "\uE100")),
                )
            val view =
                object : WindowView("w") {
                    var selected: Int? by state(null)

                    override fun WindowScope.bind() {
                        collectionItem("pets") { null }
                        collectionSelection("pets") { selected }
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            val plain = PlainTextComponentSerializer.plainText()
            val boxes = { plain.serialize(handle.titles.last()).count { it == '\uE100' } }

            boxes() shouldBe 0
            view.selected = 0
            scheduler.runAll()
            boxes() shouldBe 1
            val first = handle.titles.last()
            view.selected = 1
            scheduler.runAll()
            boxes() shouldBe 1
            handle.titles.last() shouldNotBe first
        }
    })

package com.chunkzero.window

import com.chunkzero.window.manifest.CollectionEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

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
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var selected by state(0)

                    override fun WindowScope<Any>.bind() {
                        collection(
                            "pets",
                            render = { index ->
                                if (index == selected) "diamond" else "paper"
                            },
                            handler = { clicks += it },
                        )
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)) shouldBe "diamond"
            handle.items.getValue(SlotRef(SlotArea.PLAYER, 9)) shouldBe "paper"

            handle.click(SlotRef(SlotArea.PLAYER, 9), right = true)
            clicks.single().index shouldBe 1
            clicks.single().slot shouldBe SlotRef(SlotArea.PLAYER, 9)
            clicks.single().right shouldBe true

            view.selected = 1
            scheduler.runAll()
            handle.items.getValue(SlotRef(SlotArea.CONTAINER, 0)) shouldBe "paper"
            handle.items.getValue(SlotRef(SlotArea.PLAYER, 9)) shouldBe "diamond"
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
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var selected: Int? by state(null)

                    override fun WindowScope<Any>.bind() {
                        collectionItem("pets") { null }
                        collectionSelection("pets") { selected }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()
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

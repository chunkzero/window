package dev.oglass.window

import dev.oglass.window.manifest.CollectionEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
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
    })

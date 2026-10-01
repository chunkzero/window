package dev.oglass.window

import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.ButtonDefault
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import net.minestom.server.component.DataComponents
import dev.oglass.window.manifest.ButtonTooltip as ManifestTooltip

class BindingValidationTest :
    StringSpec({
        fun manifest() =
            TestManifests.manifest(
                container = "generic_9x1",
                slots =
                    mapOf(
                        "title" to TestManifests.slot(x = 8, width = 100, align = Align.LEFT),
                        "label" to
                            TestManifests.slot(
                                x = 8,
                                width = 100,
                                align = Align.LEFT,
                                text = "Static",
                            ),
                    ),
                buttons =
                    mapOf(
                        "buy" to TestManifests.button(slots = listOf(4)),
                        "exit" to
                            TestManifests.button(slots = listOf(0), default = ButtonDefault.CLOSE),
                        "info" to
                            TestManifests.button(
                                slots = listOf(8),
                                action = false,
                                tooltip =
                                    ManifestTooltip(
                                        title = "Info",
                                        lines = listOf("Hover-only help"),
                                    ),
                            ),
                    ),
            )

        fun open(view: WindowView) {
            testSession(manifest(), "w", view, ManualScheduler(), FakeInventoryHandle()).open()
        }

        "binding an unknown slot fails fast" {
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("nope") { Component.text("x") }
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { open(view) }
            ex.message!! shouldContain "Unknown slot 'nope'"
        }

        "binding a static label is rejected" {
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        slot("label") { Component.text("nope") }
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { open(view) }
            ex.message!! shouldContain "static label"
        }

        "static labels render without a binding" {
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("Dynamic") }
                        button("buy") {}
                    }
                }
            testSession(manifest(), "w", view, ManualScheduler(), handle).open()

            val plain = PlainTextComponentSerializer.plainText().serialize(handle.titles.single())
            plain shouldContain "Static"
            plain shouldContain "Dynamic"
        }

        "binding an unknown button fails fast" {
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("ghost") {}
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { open(view) }
            ex.message!! shouldContain "Unknown button 'ghost'"
        }

        "an unbound dynamic slot is an error" {
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("buy") {}
                    }
                }
            val ex = shouldThrow<IllegalStateException> { open(view) }
            ex.message!! shouldContain "Unbound dynamic slots"
            ex.message!! shouldContain "title"
        }

        "a button with neither handler nor default is an error" {
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        // "buy" has no default and no handler -> error.
                    }
                }
            val ex = shouldThrow<IllegalStateException> { open(view) }
            ex.message!! shouldContain "neither a handler nor a default"
            ex.message!! shouldContain "buy"
        }

        "a hotspot with a tooltip needs no handler and receives a hitbox item" {
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }
                }
            testSession(manifest(), "w", view, ManualScheduler(), handle).open()

            val item = handle.items.getValue(SlotRef(SlotArea.CONTAINER, 8))
            item.get(DataComponents.ITEM_MODEL) shouldBe "window:gui/hitbox"
            PlainTextComponentSerializer
                .plainText()
                .serialize(item.get(DataComponents.CUSTOM_NAME)!!) shouldBe "Info"
            val lore = item.get(DataComponents.LORE)!!
            PlainTextComponentSerializer.plainText().serialize(lore.single()) shouldBe
                "Hover-only help"

            handle.clickContainer(8)
            handle.closed shouldBe false
        }

        "a default=close button needs no handler and routes a click to close" {
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }
                }
            val session = testSession(manifest(), "w", view, ManualScheduler(), handle)
            session.open()

            // Click on exit's slot (0) -> default close.
            handle.clickContainer(0)
            handle.closed shouldBe true
        }

        "clicks route to the matching button handler with derived modifiers" {
            val handle = FakeInventoryHandle()
            val clicks = mutableListOf<Click>()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") { clicks += it }
                    }
                }
            testSession(manifest(), "w", view, ManualScheduler(), handle).open()

            handle.clickContainer(slot = 4, shift = true, right = true)
            clicks.size shouldBe 1
            clicks[0].inventorySlot shouldBe 4
            clicks[0].area shouldBe SlotArea.CONTAINER
            clicks[0].shift shouldBe true
            clicks[0].right shouldBe true
        }

        "clicks outside any button rect are ignored" {
            val handle = FakeInventoryHandle()
            val clicks = mutableListOf<Click>()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") { clicks += it }
                    }
                }
            testSession(manifest(), "w", view, ManualScheduler(), handle).open()

            handle.clickContainer(slot = 7) // no button there
            clicks.size shouldBe 0
            handle.closed shouldBe false
        }

        "client close invokes onClose and tears down listeners" {
            var closedCalls = 0
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }

                    override fun onClose() {
                        closedCalls++
                    }
                }
            testSession(manifest(), "w", view, ManualScheduler(), handle).open()

            handle.clientClose()
            closedCalls shouldBe 1
            handle.listenersTorndown shouldBe true
        }

        "session.close is idempotent" {
            var closedCalls = 0
            val handle = FakeInventoryHandle()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }

                    override fun onClose() {
                        closedCalls++
                    }
                }
            val session = testSession(manifest(), "w", view, ManualScheduler(), handle)
            session.open()
            session.close()
            session.close()
            closedCalls shouldBe 1
        }
    })

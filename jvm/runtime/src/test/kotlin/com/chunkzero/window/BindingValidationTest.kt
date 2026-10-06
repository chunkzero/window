package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ButtonDefault
import com.chunkzero.window.manifest.TooltipEntry
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

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
                                    TooltipEntry(
                                        title = "Info",
                                        lines = listOf("Hover-only help"),
                                    ),
                            ),
                    ),
            )

        "binding an unknown slot fails fast" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        slot("nope") { Component.text("x") }
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { view.open() }
            ex.message!! shouldContain "Unknown slot 'nope'"
        }

        "binding a static label is rejected" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        slot("label") { Component.text("nope") }
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { view.open() }
            ex.message!! shouldContain "static label"
        }

        "static labels render without a binding" {
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("Dynamic") }
                        button("buy") {}
                    }
                }
            view.open()

            val plain = PlainTextComponentSerializer.plainText().serialize(handle.titles.single())
            plain shouldContain "Static"
            plain shouldContain "Dynamic"
        }

        "binding an unknown button fails fast" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("ghost") {}
                    }
                }
            val ex = shouldThrow<IllegalArgumentException> { view.open() }
            ex.message!! shouldContain "Unknown button 'ghost'"
        }

        "an unbound dynamic slot is an error" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        button("buy") {}
                    }
                }
            val ex = shouldThrow<IllegalStateException> { view.open() }
            ex.message!! shouldContain "Unbound dynamic slots"
            ex.message!! shouldContain "title"
        }

        "a button with neither handler nor default is an error" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        // "buy" has no default and no handler -> error.
                    }
                }
            val ex = shouldThrow<IllegalStateException> { view.open() }
            ex.message!! shouldContain "neither a handler nor a default"
            ex.message!! shouldContain "buy"
        }

        "a hotspot with a tooltip needs no handler and receives a hitbox item" {
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }
                }
            view.open()

            val item = handle.items.getValue(SlotRef(SlotArea.CONTAINER, 8)) as WindowItem.Hitbox
            item.model.asString() shouldBe "window:gui/hitbox"
            val tooltip = item.tooltip!!
            PlainTextComponentSerializer.plainText().serialize(tooltip.title) shouldBe "Info"
            PlainTextComponentSerializer.plainText().serialize(tooltip.lines.single()) shouldBe
                "Hover-only help"

            handle.clickContainer(8)
            handle.closed shouldBe false
        }

        "a default=close button needs no handler and routes a click to close" {
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }
                }
            val session = view.open()

            // Click on exit's slot (0) -> default close.
            handle.clickContainer(0)
            handle.closed shouldBe true
        }

        "clicks route to the matching button handler with derived modifiers" {
            val host = FakeHost()
            val handle = host.container
            val clicks = mutableListOf<Click>()
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") { clicks += it }
                    }
                }
            view.open()

            handle.clickContainer(slot = 4, shift = true, right = true)
            clicks.size shouldBe 1
            clicks[0].inventorySlot shouldBe 4
            clicks[0].area shouldBe SlotArea.CONTAINER
            clicks[0].shift shouldBe true
            clicks[0].right shouldBe true
        }

        "clicks outside any button rect are ignored" {
            val host = FakeHost()
            val handle = host.container
            val clicks = mutableListOf<Click>()
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") { clicks += it }
                    }
                }
            view.open()

            handle.clickContainer(slot = 7) // no button there
            clicks.size shouldBe 0
            handle.closed shouldBe false
        }

        "client close invokes onClose once" {
            var closedCalls = 0
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }

                    override fun onClose() {
                        closedCalls++
                    }
                }
            view.open()

            handle.clientClose()
            closedCalls shouldBe 1
            handle.closed shouldBe false
        }

        "session.close is idempotent" {
            var closedCalls = 0
            val host = FakeHost()
            val handle = host.container
            val view =
                object : TestView(manifest(), host) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }

                    override fun onClose() {
                        closedCalls++
                    }
                }
            val session = view.open()
            session.close()
            session.close()
            closedCalls shouldBe 1
            handle.closed shouldBe true
        }

        "views are single-use" {
            val view =
                object : TestView(manifest(), FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        slot("title") { Component.text("t") }
                        button("buy") {}
                    }
                }
            view.open()
            shouldThrow<IllegalStateException> { view.open() }
        }

        "unknown windows list the known ones" {
            val ex = shouldThrow<IllegalArgumentException> { WindowDefinition(manifest(), "nope") }
            ex.message!! shouldContain "known windows: [w]"
        }
    })

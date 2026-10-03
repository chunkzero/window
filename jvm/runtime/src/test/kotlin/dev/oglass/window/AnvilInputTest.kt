package dev.oglass.window

import dev.oglass.window.internal.AnvilReopenGate
import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.AnvilInputEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents

class AnvilInputTest :
    StringSpec({
        val manifest =
            TestManifests.manifest(
                container = "anvil",
                titleOrigin = listOf(60, 6),
                slots = mapOf("query_text" to TestManifests.slot(x = 60, width = 100, align = Align.LEFT)),
                inputs = mapOf("query" to AnvilInputEntry(slot = TestManifests.containerSlot(0))),
            )

        class SearchView : WindowView("w") {
            val edits = mutableListOf<String>()
            var query by state("")

            override fun WindowScope.bind() {
                slot("query_text") { Component.text(query) }
                anvilInput("query") {
                    edits += it
                    query = it
                }
            }

            fun clear() = input("query", "")
        }

        fun FakeInventoryHandle.seedName() =
            items.getValue(SlotRef(SlotArea.CONTAINER, 0)).get(DataComponents.CUSTOM_NAME)

        "static anvils deliver edits without reopening and set the edit box in place" {
            val edits = mutableListOf<String>()
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        anvilInput("query") { edits += it }
                    }

                    fun clear() = input("query", "")
                }
            val handle = FakeInventoryHandle()
            val static =
                TestManifests.manifest(
                    container = "anvil",
                    inputs = mapOf("query" to AnvilInputEntry(slot = TestManifests.containerSlot(0))),
                )
            testSession(static, "w", view, ManualScheduler(), handle).open()
            handle.input("")
            handle.input("a")
            handle.input("as")
            handle.seedName() shouldBe Component.text("as")
            view.clear()
            handle.input("")

            edits shouldBe listOf("a", "as", "")
            handle.seedName() shouldBe Component.text("")
            handle.titles.size shouldBe 1
        }

        "setting the input of a reopening anvil delivers it and reopens with it" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("ab")
            scheduler.runAll()
            handle.pong()
            view.clear()
            scheduler.runAll()

            view.edits shouldBe listOf("ab", "")
            handle.titles.size shouldBe 3
            handle.seedName() shouldBe Component.text("")
        }

        "title changes wait until the player pauses typing" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.tick()
            handle.input("di")
            scheduler.tick()
            handle.input("din")
            scheduler.tick()
            handle.titles.size shouldBe 1

            scheduler.runAll()
            handle.titles.size shouldBe 2
            view.edits shouldBe listOf("d", "di", "din")
            handle.seedName() shouldBe Component.text("din")
        }

        "reopen echoes are not edits, and edits typed on a rewound edit box are rebased" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("a")
            scheduler.runAll()
            handle.titles.size shouldBe 2

            // Typed on the old screen before the "a" reopen lands, then the reopen's echo.
            handle.input("as")
            handle.input("a")
            handle.pong()
            // Typed on the box the reopen rewound to "a".
            handle.input("ad")
            scheduler.runAll()
            handle.titles.size shouldBe 3

            handle.input("asd")
            handle.pong()
            scheduler.runAll()
            handle.titles.size shouldBe 3
            view.edits shouldBe listOf("a", "as", "asd")
            handle.seedName() shouldBe Component.text("asd")
        }

        "an edit equal to the in-flight seed is delivered, and the trailing echo is dropped" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.runAll()
            handle.input("di")
            handle.input("d")
            handle.input("d")
            handle.pong()
            scheduler.runAll()

            view.edits shouldBe listOf("d", "di", "d")
            handle.seedName() shouldBe Component.text("d")
        }

        "stale edits rebase onto the latest input around the text the edit box lost" {
            AnvilReopenGate.rebase("a", "ad", "as") shouldBe "asd"
            AnvilReopenGate.rebase("a", "", "as") shouldBe "s"
            AnvilReopenGate.rebase("a", "Xa", "as") shouldBe "Xas"
            AnvilReopenGate.rebase("ab", "a", "axb") shouldBe "ax"
            AnvilReopenGate.rebase("ab", "aXb", "ayb") shouldBe "ayXb"
            AnvilReopenGate.rebase("ab", "aZb", "aXbY") shouldBe "aXZbY"
        }

        "edits held for a pending reopen reach the view before a close" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.runAll()
            handle.input("di")
            handle.clientClose()

            view.edits shouldBe listOf("d", "di")
        }

        "a held edit that closes the window ends the client close" {
            var closes = 0
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        slot("query_text") { Component.empty() }
                        anvilInput("query") { if (it == "x") close() }
                    }

                    override fun onClose() {
                        closes++
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("x")
            handle.clientClose()

            closes shouldBe 1
        }
    })

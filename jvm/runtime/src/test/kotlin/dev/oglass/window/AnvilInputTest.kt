package dev.oglass.window

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
        }

        fun FakeInventoryHandle.seedName() =
            items.getValue(SlotRef(SlotArea.CONTAINER, 0)).get(DataComponents.CUSTOM_NAME)

        "reopen echoes are not edits, and edits typed before a reopen lands are restored" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.tick()
            handle.titles.size shouldBe 2

            // Typed on the old screen before the "d" reopen lands, then the reopen's echo.
            handle.input("di")
            handle.input("d")
            handle.pong()
            scheduler.tick()
            handle.titles.size shouldBe 3

            handle.input("di")
            handle.pong()
            scheduler.runAll()
            handle.titles.size shouldBe 3
            view.edits shouldBe listOf("d", "di")
            handle.seedName() shouldBe Component.text("di")
        }

        "an edit matching the in-flight seed is kept, and late echoes are still dropped" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.tick()
            handle.input("di")
            handle.input("d")
            repeat(40) { scheduler.tick() }
            handle.input("d")
            handle.pong()
            scheduler.runAll()

            view.edits shouldBe listOf("d", "di", "d")
            handle.seedName() shouldBe Component.text("d")
        }

        "edits held for a pending reopen reach the view before a close" {
            val view = SearchView()
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()
            handle.input("")
            handle.pong()

            handle.input("d")
            scheduler.tick()
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

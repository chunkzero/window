package dev.oglass.window

import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.AnvilInputEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents

class AnvilInputTest :
    StringSpec({
        "reopen echoes are not edits and later titles wait for them" {
            val manifest =
                TestManifests.manifest(
                    container = "anvil",
                    titleOrigin = listOf(60, 6),
                    slots = mapOf("query_text" to TestManifests.slot(x = 60, width = 100, align = Align.LEFT)),
                    inputs = mapOf("query" to AnvilInputEntry(slot = TestManifests.containerSlot(0))),
                )
            val edits = mutableListOf<String>()
            val view =
                object : WindowView("w") {
                    var query by state("")

                    override fun WindowScope.bind() {
                        slot("query_text") { Component.text(query) }
                        anvilInput("query") {
                            edits += it
                            query = it
                        }
                    }
                }
            val scheduler = ManualScheduler()
            val handle = FakeInventoryHandle()
            testSession(manifest, "w", view, scheduler, handle).open()

            handle.input("")
            handle.input("d")
            scheduler.tick()
            handle.titles.size shouldBe 2

            // Typed before the "d" reopen arrives: re-rendered, but its title waits for the echo.
            handle.input("di")
            scheduler.tick()
            handle.titles.size shouldBe 2

            handle.input("d")
            handle.titles.size shouldBe 3
            handle.input("di")
            scheduler.runAll()
            handle.titles.size shouldBe 3
            edits shouldBe listOf("d", "di")
            handle.items
                .getValue(SlotRef(SlotArea.CONTAINER, 0))
                .get(DataComponents.CUSTOM_NAME) shouldBe Component.text("di")
        }
    })

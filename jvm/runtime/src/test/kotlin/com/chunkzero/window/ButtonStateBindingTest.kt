package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.ButtonState
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class ButtonStateBindingTest :
    StringSpec({
        fun FakeContainer.model(slot: Int) =
            when (val item = items.getValue(SlotRef(SlotArea.CONTAINER, slot))) {
                is WindowItem.Hitbox -> item.model.asString()
                is WindowItem.AnvilSeed -> item.model.asString()
                else -> error("Unexpected item $item")
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
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var selected by state("best")

                    override fun WindowScope<Any>.bind() {
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
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.model(0) shouldBe "demo:gui/selected"
            handle.model(1) shouldBe "demo:gui/unselected"
            handle.clickContainer(1)
            scheduler.runAll()
            selectedValues shouldBe listOf("new")
            handle.model(0) shouldBe "demo:gui/unselected"
            handle.model(1) shouldBe "demo:gui/selected"
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
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var enabled by state(false)

                    override fun WindowScope<Any>.bind() {
                        anvilInput("query") { queries += it }
                        enabledButton("confirm", { enabled }) { confirmations++ }
                    }
                }
            val scheduler = host.scheduler
            val handle = host.container
            view.open()

            handle.model(0) shouldBe "demo:gui/search"
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

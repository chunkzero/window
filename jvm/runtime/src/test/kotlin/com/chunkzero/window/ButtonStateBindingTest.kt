package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.AnvilInputEntry
import com.chunkzero.window.manifest.HitboxEntry
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain

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
                    "selected" to HitboxEntry(itemModel = "demo:gui/selected"),
                    "unselected" to HitboxEntry(itemModel = "demo:gui/unselected"),
                )
            val best = TestManifests.stateButton("best", listOf(0), states)
            val new = TestManifests.stateButton("new", listOf(1), states)
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions = best.regions + new.regions,
                    switches = mapOf("best" to best.switch, "new" to new.switch),
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
            val confirm =
                TestManifests.stateButton(
                    "confirm",
                    listOf(2),
                    mapOf(
                        "enabled" to HitboxEntry(itemModel = "demo:gui/enabled"),
                        "disabled" to HitboxEntry(itemModel = "demo:gui/disabled"),
                    ),
                )
            val manifest =
                TestManifests.manifest(
                    container = "anvil",
                    titleOrigin = listOf(60, 6),
                    regions = confirm.regions,
                    switches = mapOf("confirm" to confirm.switch),
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

        "runtime action buttons run their action only while enabled" {
            val close =
                TestManifests.stateButton(
                    "window:close",
                    listOf(2),
                    mapOf(
                        "enabled" to HitboxEntry(itemModel = "demo:gui/enabled"),
                        "disabled" to HitboxEntry(itemModel = "demo:gui/disabled"),
                    ),
                )
            val regions =
                close.regions.mapValues { (key, region) ->
                    if (key.endsWith(".disabled")) {
                        region.copy(action = null)
                    } else {
                        region.copy(defaultAction = "window:close")
                    }
                }
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions = regions,
                    switches = mapOf("window:close" to close.switch),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var canClose by state(false)

                    override fun WindowScope<Any>.bind() {
                        buttonState("window:close") { if (canClose) "enabled" else "disabled" }
                    }
                }
            val handle = host.container
            view.open()

            handle.model(2) shouldBe "demo:gui/disabled"
            handle.clickContainer(2)
            handle.closed shouldBe false
            view.canClose = true
            host.scheduler.runAll()
            handle.model(2) shouldBe "demo:gui/enabled"
            handle.clickContainer(2)
            handle.closed shouldBe true
        }

        "runtime action buttons route clicks by the current state before the next render" {
            val close =
                TestManifests.stateButton(
                    "window:close",
                    listOf(2),
                    mapOf(
                        "enabled" to HitboxEntry(itemModel = "demo:gui/enabled"),
                        "disabled" to HitboxEntry(itemModel = "demo:gui/disabled"),
                    ),
                )
            val regions =
                close.regions.mapValues { (key, region) ->
                    if (key.endsWith(".disabled")) {
                        region.copy(action = null)
                    } else {
                        region.copy(defaultAction = "window:close")
                    }
                }
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions = regions,
                    switches = mapOf("window:close" to close.switch),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var canClose by state(false)

                    override fun WindowScope<Any>.bind() {
                        buttonState("window:close") { if (canClose) "enabled" else "disabled" }
                    }
                }
            val handle = host.container
            view.open()

            handle.model(2) shouldBe "demo:gui/disabled"
            view.canClose = true
            handle.clickContainer(2)
            handle.closed shouldBe true
        }

        "runtime action buttons ignore clicks after being disabled before the next render" {
            val close =
                TestManifests.stateButton(
                    "window:close",
                    listOf(2),
                    mapOf(
                        "enabled" to HitboxEntry(itemModel = "demo:gui/enabled"),
                        "disabled" to HitboxEntry(itemModel = "demo:gui/disabled"),
                    ),
                )
            val regions =
                close.regions.mapValues { (key, region) ->
                    if (key.endsWith(".disabled")) {
                        region.copy(action = null)
                    } else {
                        region.copy(defaultAction = "window:close")
                    }
                }
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions = regions,
                    switches = mapOf("window:close" to close.switch),
                )
            val host = FakeHost()
            val view =
                object : TestView(manifest, host) {
                    var canClose by state(false)

                    override fun WindowScope<Any>.bind() {
                        buttonState("window:close") { if (canClose) "enabled" else "disabled" }
                    }
                }
            val handle = host.container
            view.canClose = true
            view.open()

            handle.model(2) shouldBe "demo:gui/enabled"
            view.canClose = false
            handle.clickContainer(2)
            handle.closed shouldBe false
        }

        fun closeManifest(closeState: String): WindowManifest {
            val close =
                TestManifests.stateButton(
                    "window:close",
                    listOf(2),
                    mapOf("enabled" to HitboxEntry(itemModel = "demo:gui/enabled")),
                )
            val regions =
                close.regions.mapValues { (key, region) ->
                    if (key.endsWith(
                            ".$closeState",
                        )
                    ) {
                        region.copy(defaultAction = "window:close")
                    } else {
                        region.copy(action = null)
                    }
                }
            return TestManifests.manifest(
                container = "generic_9x1",
                regions = regions,
                switches = mapOf("window:close" to close.switch),
            )
        }

        "a button without a provider routes clicks by its initial case" {
            val host = FakeHost()
            val view =
                object : TestView(closeManifest("default"), host) {
                    override fun WindowScope<Any>.bind() {}
                }
            view.open()

            host.container.clickContainer(2)
            host.container.closed shouldBe true
        }

        "an imperative button state routes clicks before the next flush" {
            val host = FakeHost()
            val view =
                object : TestView(closeManifest("enabled"), host) {
                    override fun WindowScope<Any>.bind() {}

                    fun enable() = buttonState("window:close", "enabled")
                }
            view.open()

            host.container.clickContainer(2)
            host.container.closed shouldBe false
            view.enable()
            host.container.clickContainer(2)
            host.container.closed shouldBe true
        }

        "an imperative button state is rejected when a provider is bound" {
            val host = FakeHost()
            val view =
                object : TestView(closeManifest("enabled"), host) {
                    override fun WindowScope<Any>.bind() {
                        buttonState("window:close") { "enabled" }
                    }

                    fun enable() = buttonState("window:close", "enabled")
                }
            view.open()

            shouldThrow<IllegalStateException> { view.enable() }.message shouldContain "bound in bind()"
        }
    })

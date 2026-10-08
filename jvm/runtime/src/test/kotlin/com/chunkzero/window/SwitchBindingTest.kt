package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.HitboxEntry
import com.chunkzero.window.manifest.ItemEntry
import com.chunkzero.window.manifest.SwitchCaseEntry
import com.chunkzero.window.manifest.SwitchEntry
import io.kotest.assertions.throwables.shouldThrowAny
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import io.kotest.matchers.string.shouldNotContain
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class SwitchBindingTest :
    StringSpec({
        fun plain(component: Component): String = PlainTextComponentSerializer.plainText().serialize(component)

        "hud redraws the active case on update" {
            val show =
                SwitchEntry(
                    listOf(
                        SwitchCaseEntry("true", static = "BADGE", slots = listOf("sale")),
                        SwitchCaseEntry("false"),
                    ),
                )
            val manifest =
                TestManifests.hudManifest(
                    slots = mapOf("sale" to TestManifests.slot(10, 40, Align.LEFT, text = "Sale")),
                    switches = mapOf("on_sale" to show),
                )
            val view =
                object : TestHud(manifest) {
                    var onSale = false

                    override fun HudScope.bind() {
                        switch("on_sale") { onSale.toString() }
                    }
                }

            plain(view.render()) shouldNotContain "BADGE"
            plain(view.render()) shouldNotContain "Sale"
            view.onSale = true
            plain(view.render()) shouldContain "BADGE"
            plain(view.render()) shouldContain "Sale"
        }

        "a case change swaps the hitbox items and click routes of its regions" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions =
                        mapOf(
                            "a.hit" to TestManifests.region(listOf(0), action = "pick_a"),
                            "b.hit" to TestManifests.region(listOf(0, 1), "pick_b", hitbox = HitboxEntry("demo:gui/b")),
                        ),
                    switches =
                        mapOf(
                            "tab" to
                                SwitchEntry(
                                    listOf(
                                        SwitchCaseEntry("a", regions = listOf("a.hit")),
                                        SwitchCaseEntry("b", regions = listOf("b.hit")),
                                    ),
                                ),
                        ),
                )
            val host = FakeHost()
            val picks = mutableListOf<String>()
            val view =
                object : TestView(manifest, host) {
                    var tab by state("a")

                    override fun WindowScope<Any>.bind() {
                        switch("tab") { tab }
                        button("pick_a") { picks += "a" }
                        button("pick_b") { picks += "b" }
                    }
                }
            val container = host.container
            val models = { container.items.mapValues { (_, item) -> (item as WindowItem.Hitbox).model.asString() } }
            view.open()

            models() shouldBe emptyMap()
            container.clickContainer(0)
            view.tab = "b"
            host.scheduler.runAll()
            models() shouldBe
                mapOf(SlotRef(SlotArea.CONTAINER, 0) to "demo:gui/b", SlotRef(SlotArea.CONTAINER, 1) to "demo:gui/b")
            container.clickContainer(0)
            view.tab = "a"
            host.scheduler.runAll()
            models() shouldBe emptyMap()
            container.clickContainer(1)
            picks shouldBe listOf("a", "b")
        }

        "nested cases swap their items and route clicks by the cases selected at click time" {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions =
                        mapOf(
                            "a" to TestManifests.region(listOf(0), action = "a"),
                            "x" to TestManifests.region(listOf(0), action = "x"),
                            "y" to TestManifests.region(listOf(0), action = "y"),
                        ),
                    items = mapOf("coin" to ItemEntry(listOf(TestManifests.containerSlot(1)))),
                    switches =
                        mapOf(
                            "tab" to
                                SwitchEntry(
                                    listOf(
                                        SwitchCaseEntry("a", regions = listOf("a")),
                                        SwitchCaseEntry("b", switches = listOf("mode")),
                                    ),
                                ),
                            "mode" to
                                SwitchEntry(
                                    listOf(
                                        SwitchCaseEntry("x", regions = listOf("x")),
                                        SwitchCaseEntry("y", regions = listOf("y"), items = listOf("coin")),
                                    ),
                                ),
                        ),
                )
            val host = FakeHost()
            val picks = mutableListOf<String>()
            val view =
                object : TestView(manifest, host) {
                    var tab by state("a")
                    var mode by state("y")

                    override fun WindowScope<Any>.bind() {
                        switch("tab") { tab }
                        switch("mode") { mode }
                        item("coin") { "coin" }
                        for (name in listOf("a", "x", "y")) button(name) { picks += name }
                    }
                }
            val coin = SlotRef(SlotArea.CONTAINER, 1)
            view.open()

            host.container.items[coin] shouldBe null
            view.tab = "b"
            host.container.clickContainer(0)
            host.scheduler.runAll()
            host.container.items[coin] shouldBe "coin"
            view.mode = "x"
            host.container.clickContainer(0)
            host.scheduler.runAll()
            host.container.items[coin] shouldBe null
            picks shouldBe listOf("y", "x")
        }
    })

package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.HitboxEntry
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

        "one binding renders every case copy that shares it" {
            val cases =
                SwitchEntry(
                    listOf(
                        SwitchCaseEntry("good", slots = listOf("status.good")),
                        SwitchCaseEntry("bad", slots = listOf("status.bad")),
                    ),
                )
            val copy = TestManifests.slot(10, 60, Align.LEFT)
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "status.good" to copy.copy(binding = "status"),
                            "status.bad" to copy.copy(binding = "status"),
                        ),
                    switches = mapOf("kind" to cases),
                )
            val view =
                object : TestHud(manifest) {
                    var kind = "good"
                    var status = "Nice"

                    override fun HudScope.bind() {
                        switch("kind") { kind }
                        slot("status") { Component.text(status) }
                    }
                }

            plain(view.render()) shouldContain "Nice"
            view.kind = "bad"
            view.status = "Oops"
            plain(view.render()) shouldContain "Oops"
            plain(view.render()) shouldNotContain "Nice"
        }

        "one window sprite binding fills every case copy and is validated by its binding name" {
            val copy = TestManifests.spriteSlot(x = 8, y = 20, width = 16)
            val manifest =
                TestManifests.manifest(
                    spriteSlots =
                        mapOf(
                            "icon.good" to copy.copy(binding = "icon"),
                            "icon.bad" to copy.copy(binding = "icon"),
                        ),
                    sprites = mapOf("coin" to TestManifests.sprite()),
                    switches =
                        mapOf(
                            "kind" to
                                SwitchEntry(
                                    listOf(
                                        SwitchCaseEntry("good", spriteSlots = listOf("icon.good")),
                                        SwitchCaseEntry("bad", spriteSlots = listOf("icon.bad")),
                                    ),
                                ),
                        ),
                )
            val host = FakeHost()
            object : TestView(manifest, host) {
                override fun WindowScope<Any>.bind() {
                    switch("kind") { "bad" }
                    sprite("icon") { "coin" }
                }
            }.open()

            plain(host.container.titles.last()) shouldContain "\uE000"
            shouldThrowAny {
                object : TestView(manifest, FakeHost()) {
                    override fun WindowScope<Any>.bind() {
                        switch("kind") { "bad" }
                    }
                }.open()
            }.message shouldContain "[icon]"
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
    })

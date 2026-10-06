package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.types.shouldBeSameInstanceAs
import net.kyori.adventure.audience.Audience
import net.kyori.adventure.bossbar.BossBar
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class HudViewTest :
    StringSpec({
        /** The HUD's text without the fixture's static art and spacers. */
        fun text(component: Component): String =
            PlainTextComponentSerializer
                .plainText()
                .serialize(component)
                .replace("STATIC", "")
                .filter(Char::isLetter)

        fun manifest(channel: String = "actionbar") =
            TestManifests.hudManifest(
                channel = channel,
                slots = mapOf("text" to TestManifests.slot(0, 40, Align.LEFT)),
            )

        class Label(
            manifest: WindowManifest,
            var text: String,
        ) : TestHud(manifest) {
            var binds = 0

            override fun HudScope.bind() {
                binds++
                slot("text") { Component.text(text) }
            }
        }

        "render binds once, evaluates providers each time, and reuses unchanged content" {
            val hud = Label(manifest(), "A")

            val first = hud.render()
            hud.render() shouldBeSameInstanceAs first
            hud.text = "B"

            text(hud.render()) shouldBe "B"
            hud.binds shouldBe 1
        }

        class Pair(
            var first: String,
            var second: () -> String,
        ) : TestHud(
                TestManifests.hudManifest(
                    channel = "actionbar",
                    slots =
                        mapOf(
                            "first" to TestManifests.slot(0, 40, Align.LEFT),
                            "second" to TestManifests.slot(40, 40, Align.LEFT),
                        ),
                ),
            ) {
            var binds = 0

            override fun HudScope.bind() {
                binds++
                slot("first") { Component.text(first) }
                slot("second") { Component.text(second()) }
            }
        }

        "a render recovering from a provider failure publishes changes made before the failure" {
            val hud = Pair("a", { "x" })
            hud.render()
            hud.first = "b"
            hud.second = { error("boom") }
            shouldThrow<IllegalStateException> { hud.render() }
            hud.second = { "x" }

            text(hud.render()) shouldBe "bx"
        }

        "a slot whose layout fails keeps failing until its content is valid" {
            val invalid = Component.translatable("item.minecraft.stone")
            val broken =
                object : TestHud(manifest()) {
                    var content: Component = Component.text("A")

                    override fun HudScope.bind() {
                        slot("text") { content }
                    }
                }
            broken.render()
            broken.content = invalid
            repeat(2) { shouldThrow<IllegalArgumentException> { broken.render() } }
            broken.content = Component.text("B")

            text(broken.render()) shouldBe "B"
        }

        "a failed first render binds once and completes on retry" {
            val hud = Pair("a", { error("boom") })
            shouldThrow<IllegalStateException> { hud.render() }
            hud.second = { "x" }

            text(hud.render()) shouldBe "ax"
            hud.binds shouldBe 1
        }

        "a stack joins its HUDs in insertion order" {
            val first = Label(manifest(), "A")
            val second = Label(manifest(), "B")
            val stack = HudStack(HudChannel.ACTION_BAR)

            stack.add(first)
            stack.add(second)
            stack.add(first)
            text(stack.render()) shouldBe "AB"
            second.text = "C"
            text(stack.render()) shouldBe "AC"

            stack.remove(first)
            text(stack.render()) shouldBe "C"
            stack.remove(second)
            stack.isEmpty shouldBe true
            stack.render() shouldBe Component.empty()
        }

        "HUDs are only sent on the channel they were composed for" {
            val actionBar = Label(manifest(), "A")
            val bossBar = Label(manifest("bossbar"), "B")
            val bar = BossBar.bossBar(Component.empty(), 0f, BossBar.Color.WHITE, BossBar.Overlay.PROGRESS)

            text(bar.showHud(bossBar).name()) shouldBe "B"
            shouldThrow<IllegalArgumentException> { bar.showHud(actionBar) }
            shouldThrow<IllegalArgumentException> { Audience.empty().sendHud(bossBar) }
            shouldThrow<IllegalArgumentException> { HudStack(HudChannel.ACTION_BAR).add(bossBar) }
        }
    })

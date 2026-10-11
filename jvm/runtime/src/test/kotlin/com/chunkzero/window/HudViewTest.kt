package com.chunkzero.window

import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.ArtEntry
import com.chunkzero.window.manifest.HudShaderEntry
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

        class Toned(
            manifest: WindowManifest,
            var tone: String,
        ) : TestHud(manifest) {
            override fun HudScope.bind() {
                slot("text") { Component.text("A") }
                tint("tone") { tone }
            }
        }

        fun tonedManifest(shader: HudShaderEntry?) =
            TestManifests.hudManifest(
                shader = shader,
                slots =
                    mapOf(
                        "text" to
                            TestManifests
                                .slot(0, 40, Align.LEFT, shaderMarker = shader?.let { "#030001" })
                                .copy(
                                    tint = "tone",
                                    tintMarkers =
                                        shader
                                            ?.let {
                                                mapOf(
                                                    "ok" to "#040001",
                                                    "warn" to "#040002",
                                                )
                                            }.orEmpty(),
                                ),
                    ),
                art =
                    mapOf(
                        "art/0" to
                            ArtEntry(
                                "TINTED",
                                tint = "tone",
                                tintMarkers = shader?.let { mapOf("ok" to "#020003", "warn" to "#020004") }.orEmpty(),
                            ),
                        "art/1" to ArtEntry("PLAIN"),
                    ),
                tints = mapOf("tone" to mapOf("ok" to "#3ddc84", "warn" to "#ffb020")),
            )

        "a shader-placed HUD sends tinted art and text in the marker of the picked palette entry" {
            val hud = Toned(tonedManifest(HudShaderEntry(staticMarker = "#010002", sourceBottom = 59)), "ok")

            val first = hud.render()
            colorOf(first, "TINTED") shouldBe "#020003"
            colorOf(first, "A") shouldBe "#040001"
            colorOf(first, "PLAIN") shouldBe "#010002"
            hud.render() shouldBeSameInstanceAs first

            hud.tone = "warn"
            val changed = hud.render()
            colorOf(changed, "TINTED") shouldBe "#020004"
            colorOf(changed, "A") shouldBe "#040002"
            colorOf(changed, "PLAIN") shouldBe "#010002"
        }

        "a HUD without a shader sends tinted art and text in the palette color" {
            val hud = Toned(tonedManifest(null), "warn")

            val rendered = hud.render()
            colorOf(rendered, "TINTED") shouldBe "#ffb020"
            colorOf(rendered, "A") shouldBe "#ffb020"
            colorOf(rendered, "PLAIN") shouldBe "#ffffff"

            hud.tone = "missing"
            shouldThrow<IllegalArgumentException> { hud.render() }.message shouldBe
                "Unknown palette entry 'missing' for tint 'tone' in hud 'h'; known entries: [ok, warn]"
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

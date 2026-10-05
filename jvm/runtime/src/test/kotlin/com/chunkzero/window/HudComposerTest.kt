package com.chunkzero.window

import com.chunkzero.window.internal.FontRegistry
import com.chunkzero.window.internal.HudComposer
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.HudShaderEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.nulls.shouldNotBeNull
import io.kotest.matchers.shouldBe
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextColor

class HudComposerTest :
    StringSpec({
        val table = TestManifests.spacerTable()
        val fonts = FontRegistry.fromManifest(TestManifests.hudManifest())
        val hiAdvance = fonts.measure(Component.text("Hi"), "minecraft:default").advance

        fun decode(s: String): Int {
            var total = 0
            var i = 0
            while (i < s.length) {
                val cp = s.codePointAt(i)
                total += table.getValue(cp)
                i += Character.charCount(cp)
            }
            return total
        }

        "static component carries the main font and manifest string" {
            val manifest =
                TestManifests.hudManifest(
                    static = "HUD_STATIC",
                    slots = mapOf("coins" to TestManifests.slot(10, 80, Align.LEFT)),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)

            val sc = composer.staticComponent as TextComponent
            sc.content() shouldBe "HUD_STATIC"
            sc.style().font() shouldBe Key.key("window:ui")
            sc.style().color() shouldBe NamedTextColor.WHITE
            sc.style().shadowColor() shouldBe ShadowColor.none()
        }

        "composed slot overlay returns to the HUD right edge" {
            val manifest =
                TestManifests.hudManifest(
                    width = 100,
                    slots = mapOf("coins" to TestManifests.slot(10, 20, Align.CENTER, y = 2)),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)

            val rendered =
                composer.renderSlot("slot", hud.slots.getValue("coins"), Component.text("Hi"))
            rendered.shouldNotBeNull()
            val segment = rendered.component
            val component = composer.compose("h", mapOf("coins" to rendered)).component
            val children = component.children()

            val lead = children[0] as TextComponent
            val text = children[1] as TextComponent
            val trail = children[2] as TextComponent

            text.content() shouldBe "Hi"
            text.style().shadowColor() shouldBe ShadowColor.none()
            decode(lead.content()) shouldBe -84
            decode(trail.content()) shouldBe 76
            decode(lead.content()) + hiAdvance + decode(trail.content()) shouldBe 0
        }

        "shadowed HUD slot uses an explicit text shadow color" {
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "coins" to TestManifests.slot(10, 20, Align.LEFT, shadow = true, y = 2),
                        ),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)

            val rendered =
                composer.renderSlot("slot", hud.slots.getValue("coins"), Component.text("Hi"))
            rendered.shouldNotBeNull()
            val segment = rendered.component

            segment.style().shadowColor() shouldBe ShadowColor.shadowColor(0, 0, 0, 180)
            val component = composer.compose("h", mapOf("coins" to rendered)).component
            val lead = component.children()[0] as TextComponent
            lead.style().shadowColor() shouldBe ShadowColor.none()
        }

        "shader HUD uses marker colors and independent slot segments" {
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "coins" to
                                TestManifests.slot(
                                    10,
                                    20,
                                    Align.LEFT,
                                    color = "#ffff00",
                                    shaderMarker = "#120034",
                                ),
                        ),
                    shader = HudShaderEntry(staticMarker = "#010002", sourceBottom = 59),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)

            val sc = composer.staticComponent as TextComponent
            sc.style().color() shouldBe TextColor.fromHexString("#010002")

            val rendered =
                composer.renderSlot("slot", hud.slots.getValue("coins"), Component.text("Hi"))
            rendered.shouldNotBeNull()
            val segment = rendered.component
            val text = segment as TextComponent
            text.style().color() shouldBe TextColor.fromHexString("#120034")

            val component = composer.compose("h", mapOf("coins" to rendered)).component
            val children = component.children()
            val lead = children[0] as TextComponent
            val trail = children[2] as TextComponent
            decode(lead.content()) shouldBe 10
            decode(trail.content()) shouldBe -18
        }

        "shader HUD aligns by visible width but resets by advance width" {
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "coins" to
                                TestManifests.slot(10, 20, Align.RIGHT, shaderMarker = "#120034"),
                        ),
                    shader = HudShaderEntry(staticMarker = "#010002", sourceBottom = 59),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)
            val rendered =
                composer.renderSlot("slot", hud.slots.getValue("coins"), Component.text("Hi"))
            rendered.shouldNotBeNull()
            val segment = rendered.component

            val component = composer.compose("h", mapOf("coins" to rendered)).component
            val children = component.children()
            val lead = children[0] as TextComponent
            val trail = children[2] as TextComponent
            decode(lead.content()) shouldBe 23
            decode(trail.content()) shouldBe -31
        }

        "shader HUD forces marker colors through styled children" {
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "coins" to
                                TestManifests.slot(10, 20, Align.LEFT, shaderMarker = "#120034"),
                        ),
                    shader = HudShaderEntry(staticMarker = "#010002", sourceBottom = 59),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)
            val content = Component.text("H").append(Component.text("i").color(NamedTextColor.RED))
            val segment =
                composer.renderSlot("slot", hud.slots.getValue("coins"), content)!!.component as TextComponent

            segment.style().color() shouldBe TextColor.fromHexString("#120034")
            segment.children()[0].style().color() shouldBe TextColor.fromHexString("#120034")
        }

        "shader HUD preserves shadowed text while forcing marker colors" {
            val manifest =
                TestManifests.hudManifest(
                    slots =
                        mapOf(
                            "coins" to
                                TestManifests.slot(
                                    10,
                                    20,
                                    Align.LEFT,
                                    shaderMarker = "#120034",
                                    shadow = true,
                                ),
                        ),
                    shader = HudShaderEntry(staticMarker = "#010002", sourceBottom = 59),
                )
            val hud = manifest.huds.getValue("h")
            val composer = HudComposer(manifest, hud)
            val content = Component.text("H").append(Component.text("i").color(NamedTextColor.RED))
            val segment =
                composer.renderSlot("slot", hud.slots.getValue("coins"), content)!!.component as TextComponent

            segment.style().color() shouldBe TextColor.fromHexString("#120034")
            val markerShadow = ShadowColor.shadowColor(TextColor.fromHexString("#120034")!!, 180)
            segment.style().shadowColor() shouldBe markerShadow
            segment.children()[0].style().color() shouldBe TextColor.fromHexString("#120034")
            segment.children()[0].style().shadowColor() shouldBe markerShadow
        }
    })

package com.chunkzero.window

import com.chunkzero.window.internal.FontMetrics
import com.chunkzero.window.internal.FontRegistry
import com.chunkzero.window.internal.TextWidth
import com.chunkzero.window.manifest.Align
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import io.kotest.matchers.string.shouldContain
import net.kyori.adventure.text.Component

class TextMeasureTest :
    StringSpec({
        val metrics = FontMetrics("test", TestManifests.advances(), TestManifests.glyphWidths())
        val registry =
            FontRegistry(metrics, mapOf("minecraft:default" to metrics, "window:y0" to metrics))

        fun measure(component: Component): TextWidth = registry.measure(component, "minecraft:default")

        "sums advances and measures ink without the trailing cursor gap" {
            // H(6) + i(2); ink ends at i's 1px glyph after H's advance.
            metrics.measureWidths("Hi") shouldBe TextWidth(8, 7)
            metrics.measureWidths("all") shouldBe TextWidth(12, 11)
            metrics.measureWidths("Hi ") shouldBe TextWidth(12, 7)
            metrics.measureWidths(" ") shouldBe TextWidth(4, 0)
            metrics.measureWidths("A A") shouldBe TextWidth(16, 15)
        }

        "bold adds the Minecraft bold offset to cursor and ink widths" {
            metrics.measureWidths("Hi", bold = true) shouldBe TextWidth(10, 9)
            metrics.measureWidths(" ", bold = true) shouldBe TextWidth(5, 0)
        }

        "unknown character falls back to width 6" {
            metrics.measureWidths("€") shouldBe TextWidth(6, 5)
        }

        "legacy formatting codes are not measured as glyphs" {
            metrics.measureWidths("§cHi") shouldBe TextWidth(8, 7)
            metrics.measureWidths("§zHi") shouldBe TextWidth(8, 7)
            metrics.measureWidths("Hi§") shouldBe TextWidth(8, 7)
        }

        "legacy bold, color, and reset codes change measured boldness" {
            metrics.measureWidths("§LHi") shouldBe TextWidth(10, 9)
            // Bold H(7, ink 6), then the color code clears bold for i(2, ink 1).
            metrics.measureWidths("§lH§ci") shouldBe TextWidth(9, 8)
            // Reset restores the run's bold base style for i.
            metrics.measureWidths("§cH§ri", bold = true) shouldBe TextWidth(9, 8)
        }

        "legacy formatting does not leak into sibling components" {
            measure(Component.text("§lH").append(Component.text("i"))) shouldBe TextWidth(9, 8)
        }

        "unpaired surrogates measure as the replacement character" {
            val replacement =
                FontMetrics("test", TestManifests.advances() + ("�" to 9), TestManifests.glyphWidths())

            replacement.measureWidths("\uD800") shouldBe TextWidth(9, 8)
            replacement.measureWidths("\uD800H") shouldBe TextWidth(15, 14)
            replacement.measureWidths("\uDC00H") shouldBe TextWidth(15, 14)
        }

        "non-text components are rejected" {
            val error =
                shouldThrow<IllegalArgumentException> {
                    measure(Component.text("Hi").append(Component.translatable("item.minecraft.stone")))
                }
            error.message shouldContain "translatable"
            error.message shouldContain "GlobalTranslator.render"

            shouldThrow<IllegalArgumentException> { measure(Component.keybind("key.jump")) }
        }

        "left alignment starts at slot x" {
            registry.originFor(Align.LEFT, slotX = 10, width = 100, textWidth = 30) shouldBe 10
        }

        "center alignment centers within width" {
            // 10 + (100 - 30)/2 = 10 + 35 = 45
            registry.originFor(Align.CENTER, slotX = 10, width = 100, textWidth = 30) shouldBe 45
        }

        "right alignment ends at slot right edge" {
            // 10 + 100 - 30 = 80
            registry.originFor(Align.RIGHT, slotX = 10, width = 100, textWidth = 30) shouldBe 80
        }
    })

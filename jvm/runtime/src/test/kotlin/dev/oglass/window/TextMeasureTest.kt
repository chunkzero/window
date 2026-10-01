package dev.oglass.window

import dev.oglass.window.internal.FontMetrics
import dev.oglass.window.internal.FontRegistry
import dev.oglass.window.internal.TextWidth
import dev.oglass.window.manifest.Align
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe

class TextMeasureTest :
    StringSpec({
        val metrics = FontMetrics("test", TestManifests.advances(), TestManifests.glyphWidths())
        val registry =
            FontRegistry(metrics, mapOf("minecraft:default" to metrics, "window:y0" to metrics))

        "measures known characters by summing advances" {
            // "Hi" = H(6) + i(2) = 8
            metrics.measure("Hi") shouldBe 8
            // "all" = a(6) + l(3) + l(3) = 12
            metrics.measure("all") shouldBe 12
        }

        "measures visible glyph width without trailing cursor gap" {
            metrics.visualWidth("Hi") shouldBe 7
            metrics.visualWidth("Hi ") shouldBe 7
            metrics.visualWidth(" ") shouldBe 0
        }

        "measures advance and visible width in one pass" {
            metrics.measureWidths("A A") shouldBe TextWidth(16, 15)
        }

        "bold adds the Minecraft bold offset to cursor and ink widths" {
            metrics.measureWidths("Hi", bold = true) shouldBe TextWidth(10, 9)
            metrics.measureWidths(" ", bold = true) shouldBe TextWidth(5, 0)
        }

        "unknown character falls back to width 6" {
            // '€' is absent; falls back to 6.
            metrics.measure("€") shouldBe 6
            metrics.visualWidth("€") shouldBe 5
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

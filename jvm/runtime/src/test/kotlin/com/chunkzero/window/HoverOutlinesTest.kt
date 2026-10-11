package com.chunkzero.window

import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.HitboxEntry
import com.chunkzero.window.manifest.HoverEntry
import com.chunkzero.window.manifest.TooltipEntry
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

class HoverOutlinesTest :
    StringSpec({
        val hover = HoverEntry(text = "󰀊", x = -80, y = -66)

        fun shadowOf(tooltip: TooltipEntry): ShadowColor {
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions =
                        mapOf(
                            "r" to
                                TestManifests.region(listOf(0), hitbox = HitboxEntry(tooltip = tooltip), hover = hover),
                        ),
                )
            val host = FakeHost()
            object : TestView(manifest, host) {
                override fun WindowScope<Any>.bind() = Unit
            }.open()
            val item = host.container.items.getValue(SlotRef(SlotArea.CONTAINER, 0)) as WindowItem.Hitbox
            return (
                item.tooltip!!
                    .title
                    .children()
                    .first() as TextComponent
            ).shadowColor()!!
        }

        fun hitboxes(): Map<SlotRef, Any> {
            val wide = TooltipEntry(title = "Buy", lines = listOf("a".repeat(50)))
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x1",
                    regions =
                        mapOf(
                            "buy" to
                                TestManifests.region(listOf(0), hitbox = HitboxEntry(tooltip = wide), hover = hover),
                            "sell" to TestManifests.region(listOf(1), hover = hover),
                            "plain" to TestManifests.region(listOf(2)),
                        ),
                )
            val host = FakeHost()
            object : TestView(manifest, host) {
                override fun WindowScope<Any>.bind() = Unit
            }.open()
            return host.container.items
        }

        "the outline leads the tooltip title and its shadow color places it and sizes the tooltip" {
            val item = hitboxes().getValue(SlotRef(SlotArea.CONTAINER, 0)) as WindowItem.Hitbox
            val tooltip = item.tooltip!!
            val glyph = tooltip.title.children().first() as TextComponent

            glyph.content() shouldBe hover.text
            glyph.font()!!.asString() shouldBe "window:ui"
            // 50 letters of 6 px are 300 px wide: 44 in blue, 1 in the top bits of alpha over 2 lines plus 1.
            glyph.shadowColor() shouldBe ShadowColor.shadowColor(48, 62, 44, 35)
            PlainTextComponentSerializer.plainText().serialize(tooltip.title) shouldBe hover.text + "Buy"
            tooltip.lines.size shouldBe 1
            item.tooltipStyle shouldBe null
        }

        "a region without a tooltip still gets an item, whose tooltip is only the outline and draws no box" {
            val items = hitboxes()
            val item = items.getValue(SlotRef(SlotArea.CONTAINER, 1)) as WindowItem.Hitbox
            val glyph = item.tooltip!!.title as TextComponent

            glyph.content() shouldBe hover.text
            glyph.shadowColor() shouldBe ShadowColor.shadowColor(48, 62, 0, 1)
            item.tooltip!!.lines shouldBe emptyList()
            item.tooltipStyle!!.asString() shouldBe "window:hover"
            items.containsKey(SlotRef(SlotArea.CONTAINER, 2)) shouldBe false
        }

        "a tooltip the server cannot measure or that overflows the encoding gets size zero, so nothing is cut out" {
            val zero = ShadowColor.shadowColor(48, 62, 0, 1)

            shadowOf(TooltipEntry(title = "<lang:item.minecraft.paper>")) shouldBe zero
            shadowOf(TooltipEntry(title = "Loot", lines = List(29) { "line" })) shouldBe zero
            // The widest and longest tooltip that is cut out: 1800 px over 29 lines, alpha 254.
            shadowOf(TooltipEntry(title = "a".repeat(300), lines = List(28) { "line" })) shouldBe
                ShadowColor.shadowColor(48, 62, 8, 254)
        }
    })

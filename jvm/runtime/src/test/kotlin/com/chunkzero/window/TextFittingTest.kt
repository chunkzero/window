package com.chunkzero.window

import com.chunkzero.window.internal.TitleComposer
import com.chunkzero.window.manifest.Align
import com.chunkzero.window.manifest.FontMetricsEntry
import com.chunkzero.window.manifest.SlotEntry
import com.chunkzero.window.manifest.SlotLinesEntry
import com.chunkzero.window.manifest.TextOverflow
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.shouldBe
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer

/** Lowercase letters advance 6 and draw 5 wide, a space advances 4, and "…" advances 8 and draws 7 wide. */
private val metrics =
    FontMetricsEntry(
        advances = TestManifests.advances() + ("…" to 8),
        glyphWidths = TestManifests.glyphWidths() + ("…" to 7),
        boldAdvance = 1,
    )

private val fonts = listOf("window:y0", "window:l0", "window:l1", "window:l2").associateWith { metrics }

/** The drawn text runs of [component] with their effective font and color, skipping spacer runs. */
private fun runs(
    component: Component,
    parent: Style = Style.empty(),
): List<Triple<String, String?, TextColor?>> {
    val style = component.style().merge(parent, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
    val own =
        (component as TextComponent).content().takeIf { it.isNotEmpty() && style.font()?.asString() != "window:ui" }
    val self = listOfNotNull(own?.let { Triple(it, style.font()?.asString(), style.color()) })
    return self + component.children().flatMap { runs(it, style) }
}

private fun render(
    slot: SlotEntry,
    content: Component,
): Component {
    val manifest = TestManifests.manifest(slots = mapOf("name" to slot), fontMetrics = fonts)
    return TitleComposer(manifest, manifest.windows.getValue("w")).renderSlot("name", slot, content)!!.component
}

class TextFittingTest :
    StringSpec({
        val ellipsized = TestManifests.slot(8, 40, Align.LEFT).copy(overflow = TextOverflow.ELLIPSIS)

        "ellipsis truncates overflowing content and keeps the styling of the kept prefix" {
            val content = Component.text("abc", NamedTextColor.RED).append(Component.text("defgh", NamedTextColor.BLUE))

            runs(render(ellipsized, content)) shouldBe
                listOf(
                    Triple("abc", "window:y0", NamedTextColor.RED),
                    Triple("de…", "window:y0", NamedTextColor.BLUE),
                )
            runs(render(ellipsized, Component.text("abc"))).map { it.first } shouldBe listOf("abc")
        }

        "ellipsis fits the visible width exactly, measured in the legacy bold state at the cut" {
            val exact = TestManifests.slot(8, 37, Align.LEFT).copy(overflow = TextOverflow.ELLIPSIS)
            val bold = TestManifests.slot(8, 42, Align.LEFT).copy(overflow = TextOverflow.ELLIPSIS)

            runs(render(exact, Component.text("abcdefgh"))).map { it.first } shouldBe listOf("abcde…")
            runs(render(bold, Component.text("§labcdefgh"))).map { it.first } shouldBe listOf("§labcd…")
        }

        "lines wrap at spaces and center the used lines in the slot" {
            val slot =
                TestManifests.slot(8, 40, Align.LEFT, y = 20).copy(
                    overflow = TextOverflow.ELLIPSIS,
                    lines = SlotLinesEntry(2, 7, listOf("window:l0", "window:l1", "window:l2")),
                )

            runs(render(slot, Component.text("ab"))).map { it.first to it.second } shouldBe
                listOf("ab" to "window:l1")
            runs(render(slot, Component.text("abcde fghij klmno"))).map { it.first to it.second } shouldBe
                listOf("abcde" to "window:l0", "fghij…" to "window:l2")
            val ownFont = Component.text("abcde fghij").font(Key.key("window:y0"))
            runs(render(slot, ownFont)).map { it.first to it.second } shouldBe
                listOf("abcde" to "window:l0", "fghij" to "window:l2")
            shouldThrow<IllegalArgumentException> {
                render(slot, Component.text("abcde fghij").font(Key.key("window:other")))
            }
        }

        "fit shortens the value so that it and the suffix fit the slot" {
            class Score(
                width: Int = 40,
            ) : TestHud(
                    TestManifests.hudManifest(
                        slots =
                            mapOf("holder" to TestManifests.slot(0, width, Align.LEFT)),
                        fontMetrics = fonts,
                    ),
                ) {
                override fun HudScope.bind() {
                    slot("holder") { Component.empty() }
                }

                fun fitted(
                    name: String,
                    value: String,
                ) = fit(name, Component.text(value), Component.text(" 42"))
            }
            val plain = PlainTextComponentSerializer.plainText()

            plain.serialize(Score().fitted("holder", "abcdefgh")) shouldBe "ab… 42"
            plain.serialize(Score().fitted("holder", "ab")) shouldBe "ab 42"
            plain.serialize(Score(width = 10).fitted("holder", "abcdefgh")) shouldBe " 42"
            plain.serialize(Score(width = 10).fitted("holder", "")) shouldBe " 42"
            shouldThrow<IllegalArgumentException> { Score().fitted("missing", "ab") }
        }
    })

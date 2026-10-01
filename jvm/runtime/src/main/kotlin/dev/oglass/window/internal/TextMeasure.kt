package dev.oglass.window.internal

import dev.oglass.window.manifest.Align
import dev.oglass.window.manifest.FontMetricsEntry
import dev.oglass.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.TextComponent
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextDecoration
import org.slf4j.LoggerFactory

/**
 * Measures one Minecraft font's text metrics.
 *
 * Advance is the client cursor movement after rendering a glyph. Visible width is the right edge of
 * glyph ink and excludes the final inter-glyph gap. Bold adds the Minecraft bold offset to cursor
 * advance and to non-blank glyph ink; other decorations do not affect width.
 */
internal class FontMetrics(
    private val name: String,
    private val advances: Map<String, Int>,
    private val glyphWidths: Map<String, Int> = emptyMap(),
    private val boldAdvance: Int = DEFAULT_BOLD_ADVANCE,
) {
    private val warned = HashSet<Int>()

    /** Total advance width (pixels) of [text]. */
    fun measure(
        text: String,
        bold: Boolean = false,
    ): Int = measureWidths(text, bold).advance

    /** Visible ink width (pixels) of [text]. */
    fun visualWidth(
        text: String,
        bold: Boolean = false,
    ): Int = measureWidths(text, bold).visual

    /** Both widths in one pass. */
    fun measureWidths(
        text: String,
        bold: Boolean = false,
    ): TextWidth {
        var advanceTotal = 0
        var visualWidth = 0
        var i = 0
        while (i < text.length) {
            val cp = text.codePointAt(i)
            val charCount = Character.charCount(cp)
            val key = text.substring(i, i + charCount)
            val baseAdvance = advanceFor(key, cp)
            val baseGlyphWidth = glyphWidthFor(key, baseAdvance)
            val advance = baseAdvance + if (bold) boldAdvance else 0
            val glyphWidth =
                if (bold && baseGlyphWidth > 0) baseGlyphWidth + boldAdvance else baseGlyphWidth
            if (glyphWidth > 0) {
                visualWidth = maxOf(visualWidth, advanceTotal + glyphWidth)
            }
            advanceTotal += advance
            i += charCount
        }
        return TextWidth(advanceTotal, visualWidth)
    }

    private fun advanceFor(
        key: String,
        cp: Int,
    ): Int {
        val advance = advances[key]
        if (advance != null) return advance

        if (warned.add(cp)) {
            LOGGER.warn(
                "No advance width for character U+{} ('{}') in font {}; using {}",
                cp.toString(16).uppercase().padStart(4, '0'),
                key,
                name,
                DEFAULT_ADVANCE,
            )
        }
        return DEFAULT_ADVANCE
    }

    private fun glyphWidthFor(
        key: String,
        advance: Int,
    ): Int = glyphWidths[key] ?: inferredGlyphWidth(key, advance)

    private fun inferredGlyphWidth(
        key: String,
        advance: Int,
    ): Int = if (key == " ") 0 else (advance - INTER_GLYPH_GAP).coerceAtLeast(0)

    internal companion object {
        /** Fallback advance for characters absent from the manifest's table. */
        const val DEFAULT_ADVANCE: Int = 6

        /** Fallback visible glyph width for characters absent from the manifest's table. */
        const val DEFAULT_GLYPH_WIDTH: Int = 5

        /** Minecraft default-font bold cursor offset. */
        const val DEFAULT_BOLD_ADVANCE: Int = 1
        private const val INTER_GLYPH_GAP: Int = DEFAULT_ADVANCE - DEFAULT_GLYPH_WIDTH
        private val LOGGER = LoggerFactory.getLogger(FontMetrics::class.java)

        fun fromEntry(
            name: String,
            entry: FontMetricsEntry,
        ): FontMetrics = FontMetrics(name, entry.advances, entry.glyphWidths, entry.boldAdvance)
    }
}

/** Registry of measurable fonts plus component-tree measurement helpers. */
internal class FontRegistry(
    private val defaultMetrics: FontMetrics,
    private val metricsByFont: Map<String, FontMetrics>,
) {
    private val warnedFonts = HashSet<String>()

    fun measure(
        text: String,
        bold: Boolean = false,
    ): Int = defaultMetrics.measure(text, bold)

    fun visualWidth(
        text: String,
        bold: Boolean = false,
    ): Int = defaultMetrics.visualWidth(text, bold)

    fun measureWidths(
        text: String,
        bold: Boolean = false,
    ): TextWidth = defaultMetrics.measureWidths(text, bold)

    fun measure(
        component: Component,
        fallbackFont: String,
    ): TextWidth = measureComponent(component, Style.style().font(Key.key(fallbackFont)).build())

    fun originFor(
        align: Align,
        slotX: Int,
        width: Int,
        textWidth: Int,
    ): Int =
        when (align) {
            Align.LEFT -> slotX
            Align.CENTER -> slotX + (width - textWidth) / 2
            Align.RIGHT -> slotX + width - textWidth
        }

    private fun measureComponent(
        component: Component,
        parentStyle: Style,
    ): TextWidth {
        val style = component.style().merge(parentStyle, Style.Merge.Strategy.IF_ABSENT_ON_TARGET)
        var width =
            if (component is TextComponent) {
                metricsFor(style.font())
                    .measureWidths(
                        component.content(),
                        style.decoration(TextDecoration.BOLD) == TextDecoration.State.TRUE,
                    )
            } else {
                TextWidth.ZERO
            }
        for (child in component.children()) {
            width = width.append(measureComponent(child, style))
        }
        return width
    }

    private fun metricsFor(font: Key?): FontMetrics {
        val id = font?.asString()
        if (id == null) return defaultMetrics
        val metrics = metricsByFont[id]
        if (metrics != null) return metrics

        if (warnedFonts.add(id)) {
            LOGGER.warn("No metrics for font {}; using default metrics", id)
        }
        return defaultMetrics
    }

    internal companion object {
        private val LOGGER = LoggerFactory.getLogger(FontRegistry::class.java)

        fun fromManifest(manifest: WindowManifest): FontRegistry {
            val legacy =
                FontMetrics(
                    "legacy",
                    manifest.textAdvances,
                    manifest.textGlyphWidths,
                    FontMetrics.DEFAULT_BOLD_ADVANCE,
                )
            if (manifest.fontMetrics.isEmpty()) {
                return FontRegistry(legacy, mapOf("minecraft:default" to legacy))
            }
            val metrics =
                manifest.fontMetrics.mapValues { (name, entry) ->
                    FontMetrics.fromEntry(name, entry)
                }
            val defaultMetrics =
                metrics["minecraft:default"] ?: metrics.values.firstOrNull() ?: legacy
            return FontRegistry(defaultMetrics, metrics)
        }
    }
}

internal data class TextWidth(
    val advance: Int,
    val visual: Int,
) {
    fun append(next: TextWidth): TextWidth = TextWidth(advance + next.advance, maxOf(visual, advance + next.visual))

    internal companion object {
        val ZERO = TextWidth(0, 0)
    }
}

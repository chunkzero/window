package dev.oglass.window.internal

import dev.oglass.window.manifest.FontMetricsEntry
import org.slf4j.LoggerFactory
import java.util.concurrent.ConcurrentHashMap

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
    private val warned: MutableSet<Int> = ConcurrentHashMap.newKeySet()

    /**
     * Measures one literal text run that starts with [bold], decoding legacy `§` formatting codes and
     * unpaired surrogates as the client does (see [forEachRenderedCodePoint]).
     */
    fun measureWidths(
        text: String,
        bold: Boolean = false,
    ): TextWidth {
        var advanceTotal = 0
        var visualWidth = 0
        forEachRenderedCodePoint(text, bold) { codePoint, glyphBold ->
            val key = Character.toString(codePoint)
            val baseAdvance = advanceFor(key, codePoint)
            val baseGlyphWidth = glyphWidths[key] ?: inferredGlyphWidth(key, baseAdvance)
            val boldOffset = if (glyphBold) boldAdvance else 0
            if (baseGlyphWidth > 0) {
                visualWidth = maxOf(visualWidth, advanceTotal + baseGlyphWidth + boldOffset)
            }
            advanceTotal += baseAdvance + boldOffset
        }
        return TextWidth(advanceTotal, visualWidth)
    }

    private fun advanceFor(
        key: String,
        codePoint: Int,
    ): Int {
        val advance = advances[key]
        if (advance != null) return advance

        if (warned.add(codePoint)) {
            LOGGER.warn(
                "No advance width for character U+{} ('{}') in font {}; using {}",
                codePoint.toString(16).uppercase().padStart(4, '0'),
                key,
                name,
                DEFAULT_ADVANCE,
            )
        }
        return DEFAULT_ADVANCE
    }

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

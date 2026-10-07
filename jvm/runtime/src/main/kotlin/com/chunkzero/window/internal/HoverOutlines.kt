package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.manifest.ButtonEntry
import com.chunkzero.window.manifest.HoverEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/**
 * Adds a button's hover glyph to the front of its tooltip title.
 *
 * The glyph and a cancelling spacer are net-zero, so the tooltip text does not move. Its foreground is colored for
 * the core text shader to hide; its shadow color carries the button position and the tooltip size, from which the
 * shader moves the shadow onto the button and cuts the tooltip box out of it.
 */
internal class HoverOutlines(
    manifest: WindowManifest,
) {
    private val font = Key.key(manifest.font)
    private val spacers = Spacers(manifest.spacers)
    private val fonts = FontRegistry.fromManifest(manifest)

    fun decorate(
        button: ButtonEntry,
        tooltip: ButtonTooltip,
    ): ButtonTooltip {
        val hover = button.hover ?: return tooltip
        val glyph =
            Component
                .text(hover.glyph + spacers.compose(-hover.advance))
                .font(font)
                .color(HIDDEN_COLOR)
                .shadowColor(shadow(hover, tooltipWidth(tooltip), 1 + tooltip.lines.size))
                .decoration(TextDecoration.ITALIC, false)
        return tooltip.copy(
            title =
                Component
                    .text()
                    .append(glyph)
                    .append(tooltip.title)
                    .build(),
        )
    }

    /** The widest tooltip line, or the maximum when a line holds text only the client can measure. */
    private fun tooltipWidth(tooltip: ButtonTooltip): Int =
        try {
            (listOf(tooltip.title) + tooltip.lines).maxOf { fonts.measure(it, DEFAULT_FONT).advance }
        } catch (_: IllegalArgumentException) {
            MAX_WIDTH
        }

    private fun shadow(
        hover: HoverEntry,
        width: Int,
        lines: Int,
    ): ShadowColor {
        val clampedWidth = width.coerceIn(0, MAX_WIDTH)
        val alpha = lines.coerceIn(1, MAX_LINES) or ((clampedWidth shr 8) shl 5)
        return ShadowColor.shadowColor(hover.x + 128, hover.y + 128, clampedWidth and 0xFF, alpha)
    }

    private companion object {
        /** Must match the compiler's `HOVER_HIDDEN_COLOR`. */
        val HIDDEN_COLOR: TextColor = TextColor.color(0x574801)
        const val DEFAULT_FONT = "minecraft:default"
        const val MAX_WIDTH = 2047
        const val MAX_LINES = 31
    }
}

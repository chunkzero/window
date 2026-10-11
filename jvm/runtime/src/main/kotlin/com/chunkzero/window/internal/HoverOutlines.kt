package com.chunkzero.window.internal

import com.chunkzero.window.Tooltip
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.HoverEntry
import com.chunkzero.window.manifest.WindowManifest
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.TextDecoration

/**
 * Builds the hitbox items of regions that draw an outline or art while hovered.
 *
 * The hover glyph leads the tooltip title, so the client draws it exactly while the item is hovered, and the
 * pack's core text shader moves its shadow onto the region. The shadow color tells the shader where: red and green
 * are the hover's offset from the GUI center plus 128, blue and the top 3 bits of alpha the width of the tooltip's
 * text, and the low 5 bits of alpha its line count plus 1. The shader keeps the hover out of the tooltip box those
 * describe. A tooltip whose size the server cannot know exactly, or that is larger than those fields hold, is encoded
 * with size zero, which makes no cut-out.
 */
internal class HoverOutlines(
    manifest: WindowManifest,
) {
    private val font = Key.key(manifest.font)
    private val fonts = FontRegistry.fromManifest(manifest)

    /** The pack's tooltip style with transparent sprites. */
    private val hiddenStyle = Key.key(manifest.namespace, "hover")

    /** The hitbox with [model] and [hover]'s glyph; without a [tooltip] it shows no tooltip box. */
    fun hitbox(
        model: Key,
        hover: HoverEntry,
        tooltip: Tooltip?,
    ): WindowItem.Hitbox {
        val size = tooltip?.let(::measure)
        val width = size?.width ?: 0
        val alpha = ((size?.lines ?: 0) + 1) or ((width shr 8) shl 5)
        val glyph =
            Component
                .text(hover.text)
                .font(font)
                .shadowColor(ShadowColor.shadowColor(hover.x + 128, hover.y + 128, width and 0xFF, alpha))
                .decoration(TextDecoration.ITALIC, false)
        if (tooltip == null) return WindowItem.Hitbox(model, Tooltip(glyph), hiddenStyle)
        val title =
            Component
                .text()
                .append(glyph)
                .append(tooltip.title)
                .build()
        return WindowItem.Hitbox(model, tooltip.copy(title = title))
    }

    private class Size(
        val width: Int,
        val lines: Int,
    )

    /** The size of [tooltip]'s text box, or `null` when a line has text only the client can measure or it is too big. */
    private fun measure(tooltip: Tooltip): Size? {
        val lines = 1 + tooltip.lines.size
        if (lines > MAX_LINES) return null
        val width =
            try {
                (tooltip.lines + tooltip.title).maxOf { fonts.measure(it, DEFAULT_FONT).advance }
            } catch (_: IllegalArgumentException) {
                return null
            }
        return if (width > MAX_WIDTH) null else Size(width, lines)
    }

    private companion object {
        const val DEFAULT_FONT = "minecraft:default"
        const val MAX_WIDTH = 2047

        /** The most lines that keep the packed alpha below 255, which the shader reads as the hidden text copy. */
        const val MAX_LINES = 29
    }
}

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
 * Builds the hitbox items of regions with a hover outline.
 *
 * The outline glyph leads the tooltip title, so the client draws it exactly while the item is hovered, and the
 * pack's core text shader moves its shadow onto the region. The shadow color tells the shader where: red and green
 * are the region's offset from the GUI center plus 128, blue and the top 3 bits of alpha the width of the tooltip's
 * text, and the low 5 bits of alpha its line count plus 1. The shader keeps the outline out of the tooltip box those
 * describe.
 */
internal class HoverOutlines(
    manifest: WindowManifest,
) {
    private val font = Key.key(manifest.font)
    private val fonts = FontRegistry.fromManifest(manifest)

    /** The pack's tooltip style with transparent sprites. */
    private val hiddenStyle = Key.key(manifest.namespace, "hover")

    /** The hitbox with [model] and [hover]'s outline; without a [tooltip] it shows no tooltip box. */
    fun hitbox(
        model: Key,
        hover: HoverEntry,
        tooltip: Tooltip?,
    ): WindowItem.Hitbox {
        val lines = if (tooltip == null) 0 else 1 + tooltip.lines.size
        val width = if (tooltip == null) 0 else textWidth(tooltip)
        val alpha = (lines.coerceAtMost(MAX_LINES) + 1) or ((width shr 8) shl 5)
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

    /** The widest line of [tooltip], or the most the shadow color holds when a line has text only the client knows. */
    private fun textWidth(tooltip: Tooltip): Int =
        try {
            (tooltip.lines + tooltip.title).maxOf { fonts.measure(it, DEFAULT_FONT).advance }.coerceIn(0, MAX_WIDTH)
        } catch (_: IllegalArgumentException) {
            MAX_WIDTH
        }

    private companion object {
        const val DEFAULT_FONT = "minecraft:default"
        const val MAX_WIDTH = 2047
        const val MAX_LINES = 29
    }
}

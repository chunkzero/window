package com.chunkzero.window.internal

import com.chunkzero.window.manifest.SlotEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.format.ShadowColor
import net.kyori.adventure.text.format.Style
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

internal const val TEXT_HEIGHT = 8
internal const val TEXT_SHADOW_ALPHA = 180
internal val TEXT_SHADOW: ShadowColor = ShadowColor.shadowColor(0, 0, 0, TEXT_SHADOW_ALPHA)

/** Style for spacer and static runs: [font] and [color] with no shadow or decorations. */
internal fun baseStyle(
    font: String,
    color: TextColor,
): Style =
    Style
        .style()
        .font(Key.key(font))
        .color(color)
        .shadowColor(ShadowColor.none())
        .decoration(TextDecoration.BOLD, false)
        .decoration(TextDecoration.ITALIC, false)
        .decoration(TextDecoration.UNDERLINED, false)
        .decoration(TextDecoration.STRIKETHROUGH, false)
        .decoration(TextDecoration.OBFUSCATED, false)
        .build()

/** Fallback style for slot text: the slot's font, [color], shadow, and decorations. */
internal fun slotStyle(
    slot: SlotEntry,
    color: TextColor,
    shadowColor: ShadowColor = TEXT_SHADOW,
): Style =
    Style
        .style()
        .font(Key.key(slot.font))
        .color(color)
        .also { it.applyShadow(slot.shadow, shadowColor) }
        .decoration(TextDecoration.BOLD, slot.bold)
        .decoration(TextDecoration.ITALIC, slot.italic)
        .decoration(TextDecoration.UNDERLINED, slot.underlined)
        .decoration(TextDecoration.STRIKETHROUGH, slot.strikethrough)
        .decoration(TextDecoration.OBFUSCATED, slot.obfuscated)
        .build()

internal fun Style.Builder.applyShadow(
    shadow: Boolean,
    shadowColor: ShadowColor,
) {
    shadowColor(if (shadow) shadowColor else ShadowColor.none())
}

internal fun requireColor(hex: String): TextColor =
    requireNotNull(TextColor.fromHexString(hex)) { "Invalid manifest text color $hex" }

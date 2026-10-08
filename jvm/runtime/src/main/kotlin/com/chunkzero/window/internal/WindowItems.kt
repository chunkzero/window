package com.chunkzero.window.internal

import com.chunkzero.window.Tooltip
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/** The hitbox showing a user [tooltip], styled like manifest tooltips. */
internal fun WindowDefinition.tooltipHitbox(tooltip: Tooltip): WindowItem.Hitbox =
    WindowItem.Hitbox(
        hitboxModel,
        Tooltip(
            tooltip.title.withDefaults(NamedTextColor.WHITE),
            tooltip.lines.map { it.withDefaults(NamedTextColor.GRAY) },
        ),
    )

/** Applies [color] when the component has none, and turns off the item-name italics. */
internal fun Component.withDefaults(color: TextColor): Component =
    (if (color() == null) color(color) else this).decoration(TextDecoration.ITALIC, false)

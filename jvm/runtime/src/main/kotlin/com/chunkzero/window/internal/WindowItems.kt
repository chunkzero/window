package com.chunkzero.window.internal

import com.chunkzero.window.ButtonTooltip
import com.chunkzero.window.WindowDefinition
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.manifest.ButtonEntry
import net.kyori.adventure.key.Key
import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.NamedTextColor
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/** The hitbox showing a user [tooltip] on [button], styled like manifest tooltips. */
internal fun WindowDefinition.tooltipHitbox(
    button: ButtonEntry,
    tooltip: ButtonTooltip,
): WindowItem.Hitbox =
    buttonHitbox(
        button,
        hitboxModel,
        ButtonTooltip(
            tooltip.title.withDefaults(NamedTextColor.WHITE),
            tooltip.lines.map { it.withDefaults(NamedTextColor.GRAY) },
        ),
    )

/** [button]'s hitbox with [model], adding its hover outline to [tooltip] when it has one. */
internal fun WindowDefinition.buttonHitbox(
    button: ButtonEntry,
    model: Key,
    tooltip: ButtonTooltip?,
): WindowItem.Hitbox = WindowItem.Hitbox(model, tooltip?.let { hoverOutlines.decorate(button, it) })

/** Applies [color] when the component has none, and turns off the item-name italics. */
internal fun Component.withDefaults(color: TextColor): Component =
    (if (color() == null) color(color) else this).decoration(TextDecoration.ITALIC, false)

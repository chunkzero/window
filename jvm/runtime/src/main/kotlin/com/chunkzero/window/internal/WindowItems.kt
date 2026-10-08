package com.chunkzero.window.internal

import net.kyori.adventure.text.Component
import net.kyori.adventure.text.format.TextColor
import net.kyori.adventure.text.format.TextDecoration

/** Applies [color] when the component has none, and turns off the item-name italics. */
internal fun Component.withDefaults(color: TextColor): Component =
    (if (color() == null) color(color) else this).decoration(TextDecoration.ITALIC, false)

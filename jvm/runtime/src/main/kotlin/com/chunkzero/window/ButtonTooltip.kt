package com.chunkzero.window

import net.kyori.adventure.text.Component

/** Tooltip content displayed by a Window button or hotspot inventory hitbox. */
public data class ButtonTooltip(
    /** Tooltip title/name. */
    val title: Component,
    /** Additional lore lines. */
    val lines: List<Component> = emptyList(),
)

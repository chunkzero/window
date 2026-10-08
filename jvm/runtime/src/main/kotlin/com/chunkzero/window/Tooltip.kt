package com.chunkzero.window

import net.kyori.adventure.text.Component

/** Tooltip content displayed by a Window region's inventory hitbox. */
public data class Tooltip(
    /** Tooltip title/name. */
    val title: Component,
    /** Additional lore lines. */
    val lines: List<Component> = emptyList(),
)

package com.chunkzero.window.host

import net.kyori.adventure.text.Component

/** The vanilla channel a HUD is drawn on. */
public enum class HudChannel { ACTION_BAR, BOSS_BAR, SIDEBAR }

/** A HUD to show on [channel] with initial [content]. */
public data class HudDescriptor(
    val channel: HudChannel,
    val content: Component,
)

/** A HUD a [WindowHost] is showing. */
public interface HudOutput {
    /** Replaces the shown content. */
    public fun update(content: Component)

    /** Removes the HUD; the output is not used afterwards. */
    public fun hide()
}

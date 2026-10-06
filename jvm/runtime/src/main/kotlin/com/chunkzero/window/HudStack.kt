package com.chunkzero.window

import net.kyori.adventure.text.Component

/**
 * Several HUDs of one [channel] rendered as one component, in the order they were added.
 *
 * A stack holds no other state and is not thread-safe; use it from one thread at a time.
 */
public class HudStack(
    /** The channel every HUD in this stack is composed for. */
    public val channel: HudChannel,
) {
    private val members = LinkedHashSet<HudView>()

    /** Whether the stack has no HUDs. */
    public val isEmpty: Boolean
        get() = members.isEmpty()

    /**
     * Adds [hud] after the current HUDs; does nothing if it is already in the stack.
     *
     * @throws IllegalArgumentException if [hud] is not composed for [channel].
     */
    public fun add(hud: HudView) {
        requireChannel(hud.channel, channel)
        members += hud
    }

    /** Removes [hud]; does nothing if it is not in the stack. */
    public fun remove(hud: HudView) {
        members -= hud
    }

    /** Renders every HUD in the stack and joins them in order; empty when the stack is empty. */
    public fun render(): Component = members.fold(Component.empty()) { joined, hud -> joined.append(hud.render()) }
}

package com.chunkzero.window.minestom

import com.chunkzero.window.FullscreenDefinition
import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import com.chunkzero.window.minestom.internal.MinestomFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player

/**
 * The Minestom [FullscreenHost] for [player]. Hosts are equal when they share a player.
 *
 * While a full-screen window is open, the server keeps the player in spectator mode and tells only their client it is
 * in survival, so clicks and the scroll wheel reach the server. The player listens on a child of their event node, and
 * scheduled work runs on their scheduler, so change Window state for this player from the thread that ticks them.
 */
public class MinestomFullscreenHost(
    public val player: Player,
) : FullscreenHost {
    override fun open(
        definition: FullscreenDefinition,
        content: Component,
        listener: FullscreenListener,
    ): OpenFullscreen = MinestomFullscreen.open(player, definition, content, listener)

    override fun scheduleNextTick(task: Runnable) {
        player.scheduler().scheduleNextTick(task)
    }

    override fun equals(other: Any?): Boolean = other is MinestomFullscreenHost && other.player == player

    override fun hashCode(): Int = player.hashCode()
}

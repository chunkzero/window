package com.chunkzero.window.host

import com.chunkzero.window.FullscreenDefinition
import net.kyori.adventure.text.Component

/** The server side of full-screen windows for one player: takes over their view and input while one is open. */
public interface FullscreenHost {
    /**
     * Shows full-screen window [definition], drawn by the glyph runs [content], to this host's player, and reports
     * their input to [listener] until it closes.
     *
     * The player is pinned in place facing yaw and pitch 0, which the canvas centre maps to, and their HUD is hidden.
     * [content] is sent as the line of a client-only text display at their eye, as [FullscreenDefinition.canvas]
     * describes.
     *
     * @throws IllegalStateException if the player cannot show it, for example while another full-screen window is open
     *   for them. Nothing is left behind.
     */
    public fun open(
        definition: FullscreenDefinition,
        content: Component,
        listener: FullscreenListener,
    ): OpenFullscreen

    /** Runs [task] once on the next tick that serves this host's player. */
    public fun scheduleNextTick(task: Runnable)
}

/** A full-screen window a [FullscreenHost] shows to its player. */
public interface OpenFullscreen {
    /** The player's latency in milliseconds. */
    public val latencyMillis: Int

    /** Replaces the glyph runs shown. */
    public fun show(content: Component)

    /** Turns the player's view by [yaw] and [pitch] degrees from where the client has it. */
    public fun turn(
        yaw: Float,
        pitch: Float,
    )

    /**
     * Server-initiated close: stops listening, removes the display, and restores the player's game mode, abilities,
     * position, held slot, and HUD. Does not call [FullscreenListener.onClose].
     */
    public fun close()
}

/** Input from a player viewing a full-screen window. */
public interface FullscreenListener {
    /** The player turned to [yaw] and [pitch] degrees; yaw is not wrapped. */
    public fun onLook(
        yaw: Float,
        pitch: Float,
    )

    /** A left click, reported after every rotation the client sent in the same tick. */
    public fun onClick()

    /**
     * The player left the window, by sneaking or disconnecting. Listening has stopped, and the player is restored
     * unless they disconnected.
     */
    public fun onClose()
}

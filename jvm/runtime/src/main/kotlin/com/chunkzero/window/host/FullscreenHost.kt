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
     * Opens an anvil titled [title] over the canvas, with [text] in its rename field, and reports its input to
     * [listener] until it closes. The HUD stays hidden and the hotbar blank while it is open; the anvil
     * shows no items and no cost.
     *
     * Called while an anvil is open, it opens a fresh one in its place reporting to [listener], without reporting the
     * old one's end. Edits made on an earlier anvil, and the client's echo of [text], are not reported.
     *
     * @throws IllegalStateException if another container is open or an inventory listener cancelled the anvil.
     */
    public fun openTextInput(
        title: Component,
        text: String,
        listener: FullscreenTextListener,
    )

    /**
     * Server-initiated close: stops listening, closes an open anvil, removes the display, and restores the player's game
     * mode, abilities, position, held slot, and HUD. Does not call [FullscreenListener.onClose].
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

    /** A right click, reported after every rotation the client sent in the same tick. */
    public fun onRightClick()

    /**
     * The client's held slot moved by [steps], from -4 to 4 and never 0, taken the shortest way around the hotbar;
     * positive steps move right. The scroll wheel and the number keys both move it, at most once per tick.
     */
    public fun onScroll(steps: Int)

    /**
     * The player left the window, by sneaking or disconnecting. Listening has stopped, and the player is restored
     * unless they disconnected.
     */
    public fun onClose()
}

/** Input from the anvil that [OpenFullscreen.openTextInput] opened. */
public interface FullscreenTextListener {
    /** The player changed the rename field to [text]. */
    public fun onEdit(text: String)

    /** The player clicked the anvil's result slot. The anvil has closed. */
    public fun onSubmit()

    /**
     * The anvil closed without a submit: the player closed it, or another inventory replaced it. Reported on the next
     * tick, and only while the full-screen window is open.
     */
    public fun onCancel()
}

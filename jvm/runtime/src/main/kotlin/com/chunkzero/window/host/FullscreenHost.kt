package com.chunkzero.window.host

import com.chunkzero.window.ExperimentalFullscreenApi
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
     * describes. In horse mode, the host also locks the player's view and seats them on the horse that
     * [com.chunkzero.window.manifest.FullscreenCanvasEntry.horse] describes, which the pack draws the cursor from.
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

    /** Replaces the glyph runs shown; does nothing once the window has ended. */
    public fun show(content: Component)

    /** Turns the player's view by [yaw] and [pitch] degrees from where the client has it. */
    public fun turn(
        yaw: Float,
        pitch: Float,
    )

    /** Whether a [promptText] or [promptTextDialog] prompt is open and unanswered. */
    public val promptOpen: Boolean

    /**
     * Captures text the player types, starting from [initial], while the canvas stays in view; the host draws nothing
     * of the prompt, so the window draws the text itself. Reports the whole text to [onEdit] after each edit the client
     * sends, and the outcome to [onDone] exactly once, on the player's thread: the text when the player presses Escape
     * (a click on a slot or outside the keyboard may finish it too), and `null` when another prompt replaces this one,
     * a server container opens over it, or the window ends. There is no cancel. The text holds at most 50 characters,
     * and [initial] is cut to fit. The HUD stays hidden while the prompt is open.
     *
     * Both kinds of prompt share one slot: opening either replaces an open prompt of either kind. The prompt has left
     * [promptOpen] by the time [onDone] runs, so [onDone] may open the next prompt. On a closed window, or when the host
     * cannot show the prompt, this reports `null` at once.
     */
    @ExperimentalFullscreenApi
    public fun promptText(
        initial: String,
        onEdit: (String) -> Unit,
        onDone: (String?) -> Unit,
    )

    /**
     * Shows a dialog titled [title] over the canvas with a text field holding [initial] and a Confirm and a Cancel
     * button, and reports the outcome to [onDone] exactly once, on the player's thread: the field's text on Confirm,
     * `null` on Cancel, and `null` when another prompt replaces this one, a server container opens over the dialog, or
     * the window ends. The dialog closes with the button, back to the canvas, and cannot be dismissed with Escape. The
     * text holds at most 256 characters; [initial] is cut to fit. The HUD stays hidden while the dialog is open.
     *
     * Replacement and [onDone] timing are as for [promptText].
     */
    public fun promptTextDialog(
        title: Component,
        initial: String,
        onDone: (String?) -> Unit,
    )

    /**
     * Server-initiated close: stops listening, takes an open prompt off the client, removes the display, restores the
     * player's game mode, abilities, position, held slot, and HUD, and only then reports `null` to the open prompt. Does
     * not call [FullscreenListener.onClose].
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
     * The player left the window, by sneaking or disconnecting. Listening has stopped, the player is restored unless
     * they disconnected, and an open prompt has been reported `null`.
     */
    public fun onClose()
}

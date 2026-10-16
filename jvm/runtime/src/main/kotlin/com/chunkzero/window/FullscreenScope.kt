package com.chunkzero.window

import net.kyori.adventure.text.Component

/**
 * The binding surface available inside [FullscreenView.bind].
 *
 * Every dynamic slot, switch, tint, and bar of the window must be bound exactly once; static labels carrying `text`
 * render automatically and must not be bound. Every action a control names must either be bound here or have a
 * `default_action`; actions in the `window:` namespace, such as `window:close`, are run by the runtime and cannot be
 * bound. Bindings are fixed once `bind()` returns.
 */
public interface FullscreenScope {
    /**
     * Binds a dynamic slot to a [render] lambda producing its current [Component]; it is re-invoked whenever a reactive
     * state it reads changes. The text is drawn in the slot's color: colors inside the component are not kept.
     */
    public fun slot(
        name: String,
        render: () -> Component,
    )

    /**
     * Binds a switch to a [render] lambda returning the value of the case to draw. Only the active case's art, text,
     * and controls are drawn and hit.
     */
    public fun switch(
        name: String,
        render: () -> String,
    )

    /** Binds a tint to a [render] lambda returning the name of the palette entry its art and text take. */
    public fun tint(
        name: String,
        render: () -> String,
    )

    /**
     * Binds a bar to a [render] lambda returning how much of its fill to draw, from 0 to 1, rounded to whole pixel
     * columns.
     */
    public fun bar(
        name: String,
        render: () -> Float,
    )

    /** Binds the action [name] to a [handler] invoked on each click on an active control naming it. */
    public fun button(
        name: String,
        handler: (FullscreenClick) -> Unit,
    )

    /**
     * Binds [handler] to right clicks anywhere on the canvas, at the cursor. Holding the use key repeats a right click
     * every 4 ticks.
     */
    public fun rightClick(handler: (FullscreenClick) -> Unit)

    /**
     * Binds [handler] to the scroll wheel, which reaches the server as changes to the client's held hotbar slot. Each
     * change becomes a signed step count, taken the shortest way around the 9-slot hotbar: positive when the selection
     * moves right, which scrolling down does by default.
     *
     * The client sends at most one held-slot change per tick, so notches scrolled within one tick arrive as one call,
     * and more than 4 of them read as a scroll the other way. The number keys also select a slot, and the server cannot
     * tell them apart from scrolling.
     */
    public fun scroll(handler: (steps: Int) -> Unit)
}

/**
 * A click on a full-screen control.
 *
 * @property x the cursor's canvas x, in canvas pixels from the left edge.
 * @property y the cursor's canvas y, in canvas pixels from the top edge.
 */
public data class FullscreenClick(
    val x: Int,
    val y: Int,
)

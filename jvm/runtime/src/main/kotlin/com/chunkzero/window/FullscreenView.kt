package com.chunkzero.window

import com.chunkzero.window.host.FullscreenHost
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.dynamicSlot
import net.kyori.adventure.text.Component
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a full-screen window: declares slot renders and button handlers, holds reactive state, and
 * reacts to lifecycle events.
 *
 * A view shows [definition] to the player of [host] and is single-use: [open] it once. The player aims a cursor by
 * turning, clicks with the attack and use keys, scrolls, answers a [promptText] dialog, and leaves by sneaking.
 * Reactive [state] cells read inside render lambdas re-render the window when written.
 */
public abstract class FullscreenView(
    private val definition: FullscreenDefinition,
    /** The host showing this view to its player. */
    protected val host: FullscreenHost,
) {
    private var opened = false
    private var reactivity: Reactivity? = null
    private var session: FullscreenWindowSession? = null

    /**
     * Opens this view: runs [bind], shows the window, and calls [onOpen].
     *
     * A failed open still consumes the view: [onOpen] and [onClose] never run.
     *
     * @throws IllegalStateException if this view was already opened, a dynamic slot, switch, tint, or bar is unbound, an
     *   action lacks both a handler and a default, or the host could not show it.
     * @throws IllegalArgumentException for unknown names referenced in [bind].
     */
    public fun open(): FullscreenSession {
        check(!opened) { "Fullscreen view '${definition.name}' was already opened; views are single-use" }
        opened = true
        return FullscreenWindowSession(definition, this, host).also { it.open() }
    }

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a render register that target as a dependent; writes schedule a single batched re-render on the
     * next tick. Write state only on the thread that serves the host's player.
     */
    protected fun <T> state(initial: T): ReadWriteProperty<Any?, T> = LazyState(initial) { reactivity }

    /** Declares the slot renders and button handlers for this view. */
    protected abstract fun FullscreenScope.bind()

    /** Invoked after the window is shown. */
    protected open fun onOpen() {}

    /** Invoked when the window closes: by [close], a `window:close` control, the player sneaking, or disconnecting. */
    protected open fun onClose() {}

    /**
     * Closes this window and restores the player.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun close() {
        requireSession().close()
    }

    /**
     * Forces every render target to re-render on the next tick.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun refresh() {
        requireSession().refreshAll()
    }

    /**
     * Opens a dialog titled [title] over the canvas for the player to type into, starting from [initial], and calls
     * [onDone] once with the text when they confirm, or `null` when they cancel. The dialog closes with the button and
     * the canvas is back. The text holds at most 256 characters, and [initial] is cut to fit. Escape does not dismiss
     * the dialog. While it is open the HUD stays hidden and canvas input is ignored, so the cursor stays where it was.
     *
     * Opening a prompt while one is open replaces it, and the replaced prompt's [onDone] receives `null`. When a server
     * container opens over the dialog, the open prompt's [onDone] receives `null` as well, on the next tick, and the
     * window stays open. When the window closes, it receives `null` once the player is restored. [onDone] runs after its
     * prompt has ended, so it may open another one; on a closed window it receives `null` at once.
     *
     * To prompt on a click, call this from the control's click handler:
     * ```
     * override fun onRename(click: FullscreenClick) {
     *     promptText(Component.text("Rename"), name) { it?.let { name = it } }
     * }
     * ```
     *
     * @throws IllegalStateException if the view was never opened.
     */
    protected fun promptText(
        title: Component = Component.empty(),
        initial: String = "",
        onDone: (String?) -> Unit,
    ) {
        requireSession().promptText(title, initial, onDone)
    }

    /**
     * [value] shortened with an ellipsis so that it, followed by [suffix], fits the width of slot [slot] when drawn in
     * its font and style; followed by [suffix], which is kept even when it alone is wider. Content that already fits is
     * returned unchanged.
     *
     * @throws IllegalArgumentException if [slot] is not a dynamic text slot of this window.
     */
    protected fun fit(
        slot: String,
        value: Component,
        suffix: Component = Component.empty(),
    ): Component =
        definition.composer.fit(
            definition.layers.slots.dynamicSlot(slot, "fullscreen", definition.name),
            value,
            suffix,
        )

    internal fun attach(
        session: FullscreenWindowSession,
        reactivity: Reactivity,
    ) {
        this.session = session
        this.reactivity = reactivity
    }

    internal fun invokeBind(scope: FullscreenScope) {
        with(scope) { bind() }
    }

    internal fun invokeOnOpen() {
        onOpen()
    }

    internal fun invokeOnClose() {
        onClose()
    }

    private fun requireSession(): FullscreenWindowSession = checkNotNull(session) { "Fullscreen view is not open" }
}

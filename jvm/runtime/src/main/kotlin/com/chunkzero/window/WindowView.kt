package com.chunkzero.window

import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a window: declares slot renders and button handlers, holds reactive
 * state, and reacts to lifecycle events.
 *
 * A view shows [definition] to the player of [host] and is single-use: [open] it once. Subclasses
 * implement [bind] (and optionally [onOpen]/[onClose]). Reactive [state] cells read inside render
 * lambdas drive automatic re-rendering when written.
 *
 * @param I the host's native item type.
 */
public abstract class WindowView<I : Any>(
    private val definition: WindowDefinition,
    /** The host showing this view to its player. */
    protected val host: WindowHost<I>,
) {
    private var opened = false

    /** Backing reactivity engine, attached at open. `null` until then. */
    private var reactivity: Reactivity? = null

    /** The owning session, attached at open. `null` until then. */
    private var session: ContainerWindowSession<I>? = null

    /**
     * Opens this view: runs [bind], opens the container with the composed title, and calls [onOpen].
     *
     * A failed open still consumes the view: [onOpen] and [onClose] never run, and later state writes do nothing.
     *
     * @throws IllegalStateException if this view was already opened, a dynamic slot is unbound, a
     *   button lacks both a handler and a default, or the host could not open the container (for
     *   example because an open listener cancelled it).
     * @throws IllegalArgumentException for unknown names referenced in [bind].
     */
    public fun open(): WindowSession {
        check(!opened) { "Window view '${definition.name}' was already opened; views are single-use" }
        opened = true
        return windowSession(definition, this, host).also { it.open() }
    }

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a render register that target as a dependent; writes mark dependents dirty and
     * schedule a single batched re-render. State read outside rendering is inert. May be called at
     * construction time (before opening) or later. Write state only on the thread that serves the
     * host's player.
     */
    protected fun <T> state(initial: T): ReadWriteProperty<Any?, T> = LazyState(initial) { reactivity }

    /** Declares the slot renders and button handlers for this view. */
    protected abstract fun WindowScope<I>.bind()

    /** Invoked after the container is opened and seeded. */
    protected open fun onOpen() {}

    /**
     * Invoked when the window closes (by client or server) or another inventory opens over it, before
     * cleanup. It can run while the server is still closing or replacing the inventory (including when the player
     * disconnects), so opening another window from it must be deferred, e.g. to the next tick.
     */
    protected open fun onClose() {}

    /**
     * Closes this window.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun close() {
        requireSession().close()
    }

    /**
     * Forces all render targets to re-render on the next tick.
     *
     * @throws IllegalStateException if the view is not open.
     */
    protected fun refresh() {
        requireSession().refreshAll()
    }

    /** Sets or clears the item backing a button/hotspot hover region. */
    protected fun buttonItem(
        name: String,
        item: I?,
    ) {
        requireSession().setButtonItem(name, item)
    }

    /** Sets or clears the item in a dynamic item region, including a repeater cell item. */
    protected fun item(
        name: String,
        item: I?,
    ) {
        requireSession().setItem(name, item)
    }

    /** Uses one of the named manifest states declared by the Window source. */
    protected fun buttonState(
        name: String,
        state: String,
    ) {
        requireSession().setButtonState(name, state)
    }

    /**
     * Replaces the text of anvil input [name] with [value] and passes it to the input's handler. On a
     * static anvil the player's edit box updates in place.
     */
    protected fun input(
        name: String,
        value: String,
    ) {
        requireSession().setInput(name, value)
    }

    /** Sets or clears a tooltip on the invisible hitbox item for a button/hotspot. */
    protected fun tooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) {
        requireSession().setTooltip(name, tooltip)
    }

    internal fun attach(
        session: ContainerWindowSession<I>,
        reactivity: Reactivity,
    ) {
        this.session = session
        this.reactivity = reactivity
    }

    internal fun invokeBind(scope: WindowScope<I>) {
        with(scope) { bind() }
    }

    internal fun invokeOnOpen() {
        onOpen()
    }

    internal fun invokeOnClose() {
        onClose()
    }

    private fun requireSession(): ContainerWindowSession<I> = checkNotNull(session) { "Window view is not open" }
}

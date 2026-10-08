package com.chunkzero.window

import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import com.chunkzero.window.internal.dynamicSlot
import net.kyori.adventure.text.Component
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

    /**
     * Creates a [WindowList] showing [source] through [cells] cells, keyed by [key]. [step] is how far
     * previous/next move: [cells] to page, or a row width to scroll. [select] picks what is selected
     * when no explicitly selected item is in the list.
     *
     * [source] is read like a render lambda: its result is cached until a state it read changes or
     * the view is refreshed. May be called at construction time.
     *
     * @throws IllegalArgumentException if [cells] or [step] is not positive.
     */
    protected fun <T, K> list(
        cells: Int,
        key: (T) -> K,
        step: Int = cells,
        select: WindowList.Select = WindowList.Select.FIRST,
        source: () -> List<T>,
    ): WindowList<T, K> = WindowList(cells, key, step, select, source) { reactivity }

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

    /** Sets or clears the item in a dynamic item region. */
    protected fun item(
        name: String,
        item: I?,
    ) {
        requireSession().setItem(name, item)
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

    /**
     * [value] shortened with an ellipsis so that it, followed by [suffix], fits the width of slot [slot] when drawn in
     * its font and style; followed by [suffix], which is kept even when it alone is wider. Content that already fits is returned unchanged. Use it to keep a
     * suffix, such as a score, while shortening the text before it.
     *
     * [slot] is the slot's entry name, such as `name[2]` for one entry of an indexed handle.
     *
     * @throws IllegalArgumentException if [slot] is not a dynamic text slot of this window.
     */
    protected fun fit(
        slot: String,
        value: Component,
        suffix: Component = Component.empty(),
    ): Component {
        val entry = definition.entry
        return definition.composer.fit(
            entry.slots.dynamicSlot(slot, "window", definition.name),
            value,
            suffix,
        )
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

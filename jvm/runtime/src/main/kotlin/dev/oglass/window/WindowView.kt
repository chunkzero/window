package dev.oglass.window

import dev.oglass.window.internal.Reactivity
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a window: declares slot renders and button handlers, holds reactive
 * state, and reacts to lifecycle events.
 *
 * A view instance is single-use — it is bound to exactly one [WindowSession] when opened.
 * Subclasses implement [bind] (and optionally [onOpen]/[onClose]). Reactive [state] cells read
 * inside slot render lambdas drive automatic re-rendering when written.
 *
 * @param windowName the manifest window this view targets.
 */
public abstract class WindowView(
    public val windowName: String,
) {
    /** The player this view is attached to. Assigned before [onOpen]; valid for the view's life. */
    protected lateinit var player: Player
        private set

    /** Backing reactivity engine, attached at open. `null` until then. */
    private var reactivity: Reactivity? = null

    /** The owning session, attached at open. `null` until then. */
    private var session: WindowSession? = null

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a slot render register that slot as a dependent; writes mark dependents dirty
     * and schedule a single batched re-render. State created or read outside rendering is inert (it
     * simply triggers no re-render). May be called at construction time (before attachment) or
     * later; the delegate captures the engine lazily on first use after attachment.
     */
    protected fun <T> state(initial: T): ReadWriteProperty<Any?, T> = LazyState(initial)

    /** Declares the slot renders and button handlers for this view. */
    protected abstract fun WindowScope.bind()

    /** Invoked after the inventory is built and the view attached, before it is shown. */
    protected open fun onOpen() {}

    /** Invoked when the window closes (by client or server), before cleanup. */
    protected open fun onClose() {}

    /**
     * Closes this window.
     *
     * @throws IllegalStateException if the view is not attached to a session.
     */
    protected fun close() {
        requireSession().close()
    }

    /**
     * Forces all slots to re-render on the next scheduler tick.
     *
     * @throws IllegalStateException if the view is not attached to a session.
     */
    protected fun refresh() {
        requireSession().refreshAll()
    }

    /** Sets or clears the inventory item backing a button/hotspot hover region. */
    protected fun buttonItem(
        name: String,
        item: ItemStack?,
    ) {
        requireSession().setButtonItem(name, item)
    }

    /** Sets or clears the stack in a dynamic item region, including a repeater cell item. */
    protected fun item(
        name: String,
        item: ItemStack?,
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

    /** Sets or clears a tooltip on the invisible hitbox item for a button/hotspot. */
    protected fun tooltip(
        name: String,
        tooltip: ButtonTooltip?,
    ) {
        requireSession().setTooltip(name, tooltip)
    }

    // --- internal wiring used by WindowSession ---

    internal fun attach(
        player: Player,
        session: WindowSession,
        reactivity: Reactivity,
    ) {
        this.player = player
        this.session = session
        this.reactivity = reactivity
    }

    internal fun invokeBind(scope: WindowScope) {
        with(scope) { bind() }
    }

    internal fun invokeOnOpen() {
        onOpen()
    }

    internal fun invokeOnClose() {
        onClose()
    }

    private fun requireSession(): WindowSession = session ?: error("WindowView is not attached to a session")

    /**
     * A state delegate that defers to the session's [Reactivity] once attached, holding the value
     * locally until then so construction-time state works.
     */
    private inner class LazyState<T>(
        initial: T,
    ) : ReadWriteProperty<Any?, T> {
        private var delegate: ReadWriteProperty<Any?, T>? = null
        private var pending: T = initial

        private fun resolve(): ReadWriteProperty<Any?, T>? {
            if (delegate == null) {
                val engine = reactivity ?: return null
                val created = engine.state(pending)
                delegate = created
            }
            return delegate
        }

        override fun getValue(
            thisRef: Any?,
            property: kotlin.reflect.KProperty<*>,
        ): T {
            val d = resolve() ?: return pending
            return d.getValue(thisRef, property)
        }

        override fun setValue(
            thisRef: Any?,
            property: kotlin.reflect.KProperty<*>,
            value: T,
        ) {
            val d = resolve()
            if (d == null) {
                pending = value
            } else {
                d.setValue(thisRef, property, value)
            }
        }
    }
}

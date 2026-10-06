package com.chunkzero.window

import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a HUD: declares slot renders, holds reactive state, and reacts to
 * lifecycle events.
 *
 * A view shows [definition] to the player of [host] and is single-use: [show] it once.
 */
public abstract class HudView(
    private val definition: HudDefinition,
    private val host: WindowHost<*>,
) {
    private var shown = false
    private var reactivity: Reactivity? = null
    private var session: HudSession? = null

    /**
     * Shows this HUD: runs [bind], sends the first frame on the HUD's channel, and calls [onShow].
     *
     * @throws IllegalStateException if this view was already shown, or a dynamic slot is unbound.
     * @throws IllegalArgumentException for unknown names referenced in [bind].
     */
    public fun show(): HudSession {
        check(!shown) { "HUD view '${definition.name}' was already shown; views are single-use" }
        shown = true
        return HudSession(definition, this, host).also { it.show() }
    }

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a slot render register that slot as a dependent; writes mark dependents dirty
     * and schedule a single batched re-render. Write state only on the thread that serves the
     * host's player.
     */
    protected fun <T> state(initial: T): ReadWriteProperty<Any?, T> = LazyState(initial) { reactivity }

    /** Declares the slot renders for this HUD. */
    protected abstract fun HudScope.bind()

    /** Invoked after bindings are collected and the first HUD component is sent. */
    protected open fun onShow() {}

    /** Invoked when the HUD is hidden. */
    protected open fun onHide() {}

    /**
     * Hides this HUD.
     *
     * @throws IllegalStateException if the view is not shown.
     */
    protected fun hide() {
        requireSession().hide()
    }

    /**
     * Forces all dynamic slots to re-render on the next tick.
     *
     * @throws IllegalStateException if the view is not shown.
     */
    protected fun refresh() {
        requireSession().refresh()
    }

    internal fun attach(
        session: HudSession,
        reactivity: Reactivity,
    ) {
        this.session = session
        this.reactivity = reactivity
    }

    internal fun invokeBind(scope: HudScope) {
        with(scope) { bind() }
    }

    internal fun invokeOnShow() {
        onShow()
    }

    internal fun invokeOnHide() {
        onHide()
    }

    private fun requireSession(): HudSession = checkNotNull(session) { "HUD view is not shown" }
}

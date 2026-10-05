package com.chunkzero.window

import com.chunkzero.window.internal.LazyState
import com.chunkzero.window.internal.Reactivity
import net.minestom.server.entity.Player
import kotlin.properties.ReadWriteProperty

/**
 * A user-defined view over a HUD: declares slot renders, holds reactive state, and reacts to
 * lifecycle events.
 *
 * A view instance is single-use and is bound to exactly one [HudSession] when shown.
 *
 * @param hudName the manifest HUD this view targets.
 */
public abstract class HudView(
    public val hudName: String,
) {
    /** The player this view is attached to. Assigned before [onShow]. */
    protected lateinit var player: Player
        private set

    private var reactivity: Reactivity? = null
    private var session: HudSession? = null

    /**
     * Creates a reactive state delegate with the given [initial] value.
     *
     * Reads during a slot render register that slot as a dependent; writes mark dependents dirty
     * and schedule a single batched re-render.
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
     * @throws IllegalStateException if the view is not attached to a session.
     */
    protected fun hide() {
        requireSession().hide()
    }

    /**
     * Forces all dynamic slots to re-render on the next scheduler tick.
     *
     * @throws IllegalStateException if the view is not attached to a session.
     */
    protected fun refresh() {
        requireSession().refresh()
    }

    internal fun attach(
        player: Player,
        session: HudSession,
        reactivity: Reactivity,
    ) {
        this.player = player
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

    private fun requireSession(): HudSession = session ?: error("HudView is not attached to a session")
}

package com.chunkzero.window.internal

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.WindowHost
import net.kyori.adventure.text.Component
import java.lang.ref.WeakReference
import java.util.WeakHashMap
import java.util.concurrent.atomic.AtomicLong

/**
 * Combines every live action-bar HUD of one host into a single action-bar output.
 *
 * Each live HUD session strongly holds its host's [Bar]; the registry only weakly refers to it, so a bar never keeps
 * its own host (and through it the player) reachable after every HUD of that host is gone.
 */
internal object ActionbarMultiplexer {
    private val ids = AtomicLong()
    private val bars = WeakHashMap<WindowHost<*>, WeakReference<Bar>>()

    fun nextId(): Long = ids.incrementAndGet()

    /** Shows [component] as HUD [id]'s part of [host]'s action bar; the caller must keep the returned bar. */
    @Synchronized
    fun send(
        host: WindowHost<*>,
        id: Long,
        component: Component,
    ): Bar {
        val bar = bars[host]?.get()
        if (bar == null) {
            val components = linkedMapOf(id to component)
            return Bar(host.showHud(HudDescriptor(HudChannel.ACTION_BAR, combine(components.values))), components)
                .also { bars[host] = WeakReference(it) }
        }
        bar.components[id] = component
        bar.output.update(combine(bar.components.values))
        return bar
    }

    /** Removes HUD [id]'s part of [host]'s action bar [bar], hiding the bar with its last HUD. */
    @Synchronized
    fun hide(
        host: WindowHost<*>,
        bar: Bar,
        id: Long,
    ) {
        if (bar.components.remove(id) == null) return
        if (bar.components.isEmpty()) {
            if (bars[host]?.get() === bar) bars.remove(host)
            bar.output.hide()
        } else {
            bar.output.update(combine(bar.components.values))
        }
    }

    private fun combine(values: Collection<Component>): Component = values.fold(Component.empty(), Component::append)

    class Bar internal constructor(
        val output: HudOutput,
        val components: LinkedHashMap<Long, Component>,
    )
}

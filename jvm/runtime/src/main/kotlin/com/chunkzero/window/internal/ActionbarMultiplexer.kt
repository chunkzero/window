package com.chunkzero.window.internal

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.WindowHost
import net.kyori.adventure.text.Component
import java.util.WeakHashMap
import java.util.concurrent.atomic.AtomicLong

/**
 * Combines every live action-bar HUD of one host into a single action-bar output.
 *
 * The registry holds each bar strongly for as long as its host is reachable, so a shown HUD stays part of the merged
 * bar even when nothing else refers to its view or session. Its [HudOutput] must therefore not strongly reference the
 * host or the player that holds it (see [WindowHost.showHud]); otherwise the entry would keep its own key alive.
 */
internal object ActionbarMultiplexer {
    private val ids = AtomicLong()
    private val bars = WeakHashMap<WindowHost<*>, Bar>()

    fun nextId(): Long = ids.incrementAndGet()

    /** Shows [component] as HUD [id]'s part of [host]'s action bar and returns that bar. */
    @Synchronized
    fun send(
        host: WindowHost<*>,
        id: Long,
        component: Component,
    ): Bar {
        val bar = bars[host]
        if (bar == null) {
            val components = linkedMapOf(id to component)
            return Bar(host.showHud(HudDescriptor(HudChannel.ACTION_BAR, combine(components.values))), components)
                .also { bars[host] = it }
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
            if (bars[host] === bar) bars.remove(host)
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

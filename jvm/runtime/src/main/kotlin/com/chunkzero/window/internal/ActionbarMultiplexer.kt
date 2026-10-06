package com.chunkzero.window.internal

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.WindowHost
import net.kyori.adventure.text.Component
import java.util.WeakHashMap
import java.util.concurrent.atomic.AtomicLong

/** Combines every live action-bar HUD of one host into a single action-bar output. */
internal object ActionbarMultiplexer {
    private val ids = AtomicLong()
    private val bars = WeakHashMap<WindowHost<*>, Bar>()

    fun nextId(): Long = ids.incrementAndGet()

    @Synchronized
    fun send(
        host: WindowHost<*>,
        id: Long,
        component: Component,
    ) {
        val bar = bars[host]
        if (bar == null) {
            val components = linkedMapOf(id to component)
            bars[host] = Bar(host.showHud(HudDescriptor(HudChannel.ACTION_BAR, combine(components.values))), components)
        } else {
            bar.components[id] = component
            bar.output.update(combine(bar.components.values))
        }
    }

    @Synchronized
    fun hide(
        host: WindowHost<*>,
        id: Long,
    ) {
        val bar = bars[host] ?: return
        if (bar.components.remove(id) == null) return
        if (bar.components.isEmpty()) {
            bars.remove(host)
            bar.output.hide()
        } else {
            bar.output.update(combine(bar.components.values))
        }
    }

    private fun combine(values: Collection<Component>): Component = values.fold(Component.empty(), Component::append)

    private class Bar(
        val output: HudOutput,
        val components: LinkedHashMap<Long, Component>,
    )
}

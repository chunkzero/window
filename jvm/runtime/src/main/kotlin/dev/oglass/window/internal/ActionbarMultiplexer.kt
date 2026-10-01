package dev.oglass.window.internal

import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import java.util.WeakHashMap
import java.util.concurrent.atomic.AtomicLong

/** Combines multiple live actionbar HUDs into one packet per player. */
internal object ActionbarMultiplexer {
    private val ids = AtomicLong()
    private val components = WeakHashMap<Player, LinkedHashMap<Long, Component>>()

    fun nextId(): Long = ids.incrementAndGet()

    @Synchronized
    fun send(
        player: Player,
        id: Long,
        component: Component,
    ) {
        val playerComponents = components.getOrPut(player) { linkedMapOf() }
        playerComponents[id] = component
        player.sendActionBar(combine(playerComponents.values))
    }

    @Synchronized
    fun hide(
        player: Player,
        id: Long,
    ) {
        val playerComponents = components[player] ?: return
        playerComponents.remove(id)
        if (playerComponents.isEmpty()) {
            components.remove(player)
            player.sendActionBar(Component.empty())
        } else {
            player.sendActionBar(combine(playerComponents.values))
        }
    }

    private fun combine(values: Collection<Component>): Component {
        var out = Component.empty()
        for (component in values) {
            out = out.append(component)
        }
        return out
    }
}

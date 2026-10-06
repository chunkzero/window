package com.chunkzero.window

import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.event.Event
import net.minestom.server.event.EventNode
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType

/**
 * The server operations Window needs from its host, which differ between Minestom servers.
 *
 * Use `MinestomPlatform` from `window-minestom` for a single Minestom server, or
 * `MultistomPlatform` from `window-multistom` when players belong to separate server processes.
 */
public interface WindowPlatform {
    /**
     * Creates an unopened inventory of [type] titled [title] for [player].
     *
     * Must return an `AnvilInventory` for [InventoryType.ANVIL] so the server reports anvil text
     * input for it.
     */
    public fun createInventory(
        player: Player,
        type: InventoryType,
        title: Component,
    ): Inventory

    /** The root event node that receives [player]'s inventory, packet, and anvil events. */
    public fun eventRoot(player: Player): EventNode<Event>

    /** Runs [task] once on the next tick of the server serving [player]. */
    public fun scheduleNextTick(
        player: Player,
        task: Runnable,
    )
}

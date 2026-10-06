package com.chunkzero.window.multistom

import com.chunkzero.window.WindowPlatform
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.event.Event
import net.minestom.server.event.EventNode
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType
import net.minestom.server.inventory.type.AnvilInventory

/** [WindowPlatform] for Multistom, using the server process that owns each player. */
public object MultistomPlatform : WindowPlatform {
    override fun createInventory(
        player: Player,
        type: InventoryType,
        title: Component,
    ): Inventory =
        if (type == InventoryType.ANVIL) {
            AnvilInventory(player.process(), title)
        } else {
            Inventory(player.process(), type, title)
        }

    override fun eventRoot(player: Player): EventNode<Event> = player.process().eventHandler()

    override fun scheduleNextTick(
        player: Player,
        task: Runnable,
    ) {
        player.process().schedulerManager().scheduleNextTick(task)
    }
}

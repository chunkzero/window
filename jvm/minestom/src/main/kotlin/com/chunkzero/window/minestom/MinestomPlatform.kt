package com.chunkzero.window.minestom

import com.chunkzero.window.WindowPlatform
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.entity.Player
import net.minestom.server.event.Event
import net.minestom.server.event.EventNode
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType
import net.minestom.server.inventory.type.AnvilInventory

/** [WindowPlatform] for a single Minestom server, using its global event handler and scheduler. */
public object MinestomPlatform : WindowPlatform {
    override fun createInventory(
        player: Player,
        type: InventoryType,
        title: Component,
    ): Inventory = if (type == InventoryType.ANVIL) AnvilInventory(title) else Inventory(type, title)

    override fun eventRoot(player: Player): EventNode<Event> = MinecraftServer.getGlobalEventHandler()

    override fun scheduleNextTick(
        player: Player,
        task: Runnable,
    ) {
        MinecraftServer.getSchedulerManager().scheduleNextTick(task)
    }
}

package com.chunkzero.window.minestom.internal

import net.minestom.server.entity.Player
import net.minestom.server.inventory.Inventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.server.play.SetSlotPacket

/**
 * Temporarily takes over [player]'s own inventory slots for window controls.
 *
 * The first [setItem] snapshots and clears every inner slot; [restore] puts the snapshot back. Each change is mirrored
 * to the player region of the open [screen], since player inventory updates only refresh the player's own window.
 */
internal class PlayerSlotLease(
    private val player: Player,
    private val screen: Inventory,
) {
    private var snapshot: List<ItemStack>? = null

    fun setItem(
        slot: Int,
        item: ItemStack,
    ) {
        if (snapshot == null) {
            snapshot = (0 until player.inventory.innerSize).map(player.inventory::getItemStack)
            for (index in 0 until player.inventory.innerSize) write(index, ItemStack.AIR)
        }
        write(slot, item)
    }

    fun restore() {
        val items = snapshot ?: return
        snapshot = null
        items.forEachIndexed(::write)
    }

    private fun write(
        slot: Int,
        item: ItemStack,
    ) {
        player.inventory.setItemStack(slot, item)
        val windowSlot =
            when (slot) {
                in 0..8 -> screen.size + 27 + slot
                in 9 until player.inventory.innerSize -> screen.size + slot - 9
                else -> return
            }
        player.sendPacket(SetSlotPacket(screen.windowId.toInt(), 0, windowSlot.toShort(), item))
    }
}

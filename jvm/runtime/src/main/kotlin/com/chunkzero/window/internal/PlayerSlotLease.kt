package com.chunkzero.window.internal

import net.minestom.server.entity.Player
import net.minestom.server.inventory.Inventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.server.play.SetSlotPacket

/**
 * Temporarily takes over a player's own inventory slots for window controls.
 *
 * The first [setItem] snapshots and clears every inner slot; [restore] puts the snapshot back.
 * Changes are mirrored to the client's open window, which [openInventory] supplies.
 */
internal class PlayerSlotLease(
    private val player: Player,
    private val openInventory: () -> Inventory,
) {
    private val snapshots = LinkedHashMap<Int, ItemStack>()
    private var taken = false

    fun setItem(
        slot: Int,
        item: ItemStack,
    ) {
        takeOver()
        player.inventory.setItemStack(slot, item)
        refresh(slot, item)
    }

    fun restore() {
        if (!taken) return
        for ((slot, item) in snapshots) {
            player.inventory.setItemStack(slot, item)
            refresh(slot, item)
        }
        snapshots.clear()
        taken = false
    }

    private fun takeOver() {
        if (taken) return
        taken = true
        for (slot in 0 until player.inventory.innerSize) {
            snapshots[slot] = player.inventory.getItemStack(slot)
            player.inventory.setItemStack(slot, ItemStack.AIR)
            refresh(slot, ItemStack.AIR)
        }
    }

    private fun refresh(
        slot: Int,
        item: ItemStack,
    ) {
        val inventory = openInventory()
        val windowSlot = openWindowSlot(inventory, slot) ?: return
        player.sendPacket(SetSlotPacket(inventory.windowId.toInt(), 0, windowSlot, item))
    }

    private fun openWindowSlot(
        inventory: Inventory,
        slot: Int,
    ): Short? =
        when (slot) {
            in 0..8 -> (inventory.size + 27 + slot).toShort()
            in 9 until player.inventory.innerSize -> (inventory.size + slot - 9).toShort()
            else -> null
        }
}

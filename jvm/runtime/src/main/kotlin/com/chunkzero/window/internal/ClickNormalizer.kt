package com.chunkzero.window.internal

import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import net.minestom.server.entity.Player
import net.minestom.server.event.inventory.InventoryPreClickEvent
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.click.Click as MinestomClick

/** Maps raw Minestom click events on the open window or the player inventory to Window slots. */
internal class ClickNormalizer(
    private val player: Player,
    private val openInventory: () -> Inventory,
) {
    /** The clicked slot as a typed [SlotRef], or `null` when it is outside any control area. */
    fun slot(event: InventoryPreClickEvent): SlotRef? {
        val inventory = openInventory()
        val slot = event.slot
        if (slot < 0) return null

        return when {
            event.inventory === inventory && slot < inventory.size -> {
                SlotRef(SlotArea.CONTAINER, slot)
            }

            event.inventory === inventory -> {
                playerSlot(slot - inventory.size)
            }

            event.inventory === player.inventory -> {
                playerSlot(slot)
            }

            else -> {
                null
            }
        }
    }

    fun click(
        click: MinestomClick,
        slot: SlotRef,
    ): ClickInfo =
        when (click) {
            is MinestomClick.Left -> ClickInfo(slot, shift = false, right = false)
            is MinestomClick.Right -> ClickInfo(slot, shift = false, right = true)
            is MinestomClick.LeftShift -> ClickInfo(slot, shift = true, right = false)
            is MinestomClick.RightShift -> ClickInfo(slot, shift = true, right = true)
            else -> ClickInfo(slot, shift = false, right = false)
        }

    private fun playerSlot(index: Int): SlotRef? =
        if (index in 0 until player.inventory.innerSize) SlotRef(SlotArea.PLAYER, index) else null
}

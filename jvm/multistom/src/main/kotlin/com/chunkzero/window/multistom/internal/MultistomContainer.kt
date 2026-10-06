package com.chunkzero.window.multistom.internal

import com.chunkzero.window.SlotArea
import com.chunkzero.window.SlotRef
import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.OpenContainer
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.event.EventFilter
import net.minestom.server.event.EventListener
import net.minestom.server.event.EventNode
import net.minestom.server.event.inventory.InventoryCloseEvent
import net.minestom.server.event.inventory.InventoryOpenEvent
import net.minestom.server.event.inventory.InventoryPreClickEvent
import net.minestom.server.event.player.PlayerAnvilInputEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.trait.PlayerEvent
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType
import net.minestom.server.inventory.click.Click
import net.minestom.server.inventory.type.AnvilInventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.BundlePacket

/**
 * An [inventory] open for [player], reporting its input through a child of the player's event node.
 *
 * Opening any other inventory over this one ends it like a client close, because Multistom replaces an open inventory
 * without an [InventoryCloseEvent].
 */
internal class MultistomContainer private constructor(
    private val player: Player,
    private val inventory: Inventory,
) : OpenContainer<ItemStack> {
    private val playerSlots = PlayerSlotLease(player, inventory)
    private var node: EventNode<PlayerEvent>? = null

    override val size: Int
        get() = inventory.size

    override fun setTitle(title: Component) {
        inventory.setTitle(title)
    }

    override fun setItem(
        slot: SlotRef,
        item: ItemStack?,
    ) = when (slot.area) {
        SlotArea.CONTAINER -> inventory.setItemStack(slot.index, item ?: ItemStack.AIR)
        SlotArea.PLAYER -> playerSlots.setItem(slot.index, item ?: ItemStack.AIR)
    }

    override fun stageItem(
        slot: Int,
        item: ItemStack?,
    ) {
        inventory.setItemStack(slot, item ?: ItemStack.AIR, false)
    }

    override fun restorePlayerSlots() = playerSlots.restore()

    override fun batch(action: () -> Unit) {
        player.sendPacket(BundlePacket())
        try {
            action()
        } finally {
            player.sendPacket(BundlePacket())
        }
    }

    override fun ping(id: Int) {
        player.sendPacket(PingPacket(id))
    }

    override fun close() {
        if (!stopListening()) return
        playerSlots.restore()
        player.closeInventory()
    }

    private fun listen(listener: ContainerListener) {
        val node = EventNode.type("window-container-${inventory.windowId}", EventFilter.PLAYER)
        node.addListener(
            EventListener
                .builder(InventoryPreClickEvent::class.java)
                .ignoreCancelled(false)
                .handler { event -> click(event, listener) }
                .build(),
        )
        node.addListener(InventoryCloseEvent::class.java) { event ->
            if (event.inventory === inventory) end(listener)
        }
        node.addListener(InventoryOpenEvent::class.java) { event ->
            if (event.inventory !== inventory && player.openInventory === inventory) end(listener)
        }
        node.addListener(PlayerAnvilInputEvent::class.java) { event ->
            if (event.inventory === inventory) listener.onAnvilInput(event.input)
        }
        node.addListener(PlayerPacketEvent::class.java) { event ->
            val packet = event.packet
            if (packet is ClientPongPacket) listener.onPong(packet.id())
        }
        player.eventNode().addChild(node)
        this.node = node
    }

    /** Cancels every click on this container or the player's inventory, and reports those on a Window slot. */
    private fun click(
        event: InventoryPreClickEvent,
        listener: ContainerListener,
    ) {
        if (event.inventory !== inventory && event.inventory !== player.inventory) return
        event.isCancelled = true
        val slot = slot(event) ?: return
        val click = event.click
        listener.onClick(
            slot,
            shift = click is Click.LeftShift || click is Click.RightShift,
            right = click is Click.Right || click is Click.RightShift,
        )
    }

    private fun slot(event: InventoryPreClickEvent): SlotRef? {
        val slot = event.slot
        return when {
            slot < 0 -> null
            event.inventory !== inventory -> playerSlot(slot)
            slot < inventory.size -> SlotRef(SlotArea.CONTAINER, slot)
            else -> playerSlot(slot - inventory.size)
        }
    }

    private fun playerSlot(index: Int): SlotRef? =
        if (index in 0 until player.inventory.innerSize) SlotRef(SlotArea.PLAYER, index) else null

    /** Ends this container after the client closed or replaced it. */
    private fun end(listener: ContainerListener) {
        if (!stopListening()) return
        playerSlots.restore()
        listener.onClose()
    }

    /** Removes the event node; false when it was already removed. */
    private fun stopListening(): Boolean {
        val node = node ?: return false
        this.node = null
        player.eventNode().removeChild(node)
        return true
    }

    companion object {
        /** Opens [kind] for [player], then listens, so the container's own open event is not taken as a replacement. */
        fun open(
            player: Player,
            kind: ContainerKind,
            title: Component,
            listener: ContainerListener,
        ): MultistomContainer {
            val inventory =
                when (kind) {
                    ContainerKind.ANVIL -> AnvilInventory(player.process(), title)
                    else -> Inventory(player.process(), chestType(kind), title)
                }
            player.openInventory(inventory)
            return MultistomContainer(player, inventory).also { it.listen(listener) }
        }

        private fun chestType(kind: ContainerKind): InventoryType =
            when (kind) {
                ContainerKind.CHEST_1_ROW -> InventoryType.CHEST_1_ROW
                ContainerKind.CHEST_2_ROW -> InventoryType.CHEST_2_ROW
                ContainerKind.CHEST_3_ROW -> InventoryType.CHEST_3_ROW
                ContainerKind.CHEST_4_ROW -> InventoryType.CHEST_4_ROW
                ContainerKind.CHEST_5_ROW -> InventoryType.CHEST_5_ROW
                ContainerKind.CHEST_6_ROW -> InventoryType.CHEST_6_ROW
                ContainerKind.ANVIL -> InventoryType.ANVIL
            }
    }
}

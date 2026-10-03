package dev.oglass.window.internal

import dev.oglass.window.SlotArea
import dev.oglass.window.SlotRef
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
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
import net.minestom.server.inventory.type.AnvilInventory
import net.minestom.server.item.ItemStack
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.BundlePacket
import org.slf4j.LoggerFactory
import java.util.concurrent.atomic.AtomicLong

/**
 * Live Minestom-backed [InventoryHandle].
 *
 * Builds a real [Inventory] of the given [type], wires a per-session [EventNode] filtered to the
 * player onto the global handler, normalises [InventoryPreClickEvent] clicks (always cancelled),
 * and updates the title via [Inventory.setTitle]. Opening any other inventory over this one ends the
 * session like a close, because Minestom replaces it without an [InventoryCloseEvent].
 */
internal class LiveInventoryHandle(
    private val player: Player,
    private val type: InventoryType,
) : InventoryHandle {
    /** The backing inventory, valid after [open]. */
    lateinit var inventory: Inventory
        private set

    override val containerId: Int?
        get() = if (::inventory.isInitialized) inventory.windowId.toInt() and 0xff else null

    private var node: EventNode<PlayerEvent>? = null
    private val playerSlots = PlayerSlotLease(player) { inventory }
    private val clicks = ClickNormalizer(player) { inventory }

    override fun open(title: Component) {
        inventory =
            if (type == InventoryType.ANVIL) {
                AnvilInventory(title)
            } else {
                Inventory(type, title)
            }
        player.openInventory(inventory)
    }

    override fun setTitle(title: Component) {
        inventory.setTitle(title)
    }

    override fun setItem(
        slot: SlotRef,
        item: ItemStack,
    ) {
        when (slot.area) {
            SlotArea.CONTAINER -> {
                inventory.setItemStack(slot.index, item)
            }

            SlotArea.PLAYER -> {
                playerSlots.setItem(slot.index, item)
            }
        }
    }

    override fun stageItem(
        slot: SlotRef,
        item: ItemStack,
    ) {
        require(slot.area == SlotArea.CONTAINER) { "Only container slots can be staged" }
        inventory.setItemStack(slot.index, item, false)
    }

    override fun ping(id: Int) {
        player.sendPacket(PingPacket(id))
    }

    override fun bundle(action: () -> Unit) {
        player.sendPacket(BundlePacket())
        try {
            action()
        } finally {
            player.sendPacket(BundlePacket())
        }
    }

    override fun registerListeners(
        onClick: (ClickInfo) -> Unit,
        onClose: () -> Unit,
        onInput: (String) -> Unit,
        onPong: (Int) -> Unit,
    ) {
        val sessionNode =
            EventNode.value(
                "window-session-${NODE_ID.getAndIncrement()}",
                EventFilter.PLAYER,
                { it === player },
            )
        sessionNode.addListener(
            EventListener
                .builder(InventoryPreClickEvent::class.java)
                .ignoreCancelled(false)
                .handler { event ->
                    if (event.inventory !== inventory && event.inventory !== player.inventory) return@handler
                    event.isCancelled = true
                    val slot = clicks.slot(event) ?: return@handler
                    LOGGER.trace(
                        "Window click rawInventory=open:{} player:{} rawSlot:{} normalised={}:{}",
                        event.inventory === inventory,
                        event.inventory === player.inventory,
                        event.slot,
                        slot.area,
                        slot.index,
                    )
                    onClick(clicks.click(event.click, slot))
                }.build(),
        )
        sessionNode.addListener(InventoryCloseEvent::class.java) { event ->
            if (event.inventory !== inventory) return@addListener
            playerSlots.restore()
            onClose()
        }
        sessionNode.addListener(InventoryOpenEvent::class.java) { event ->
            if (event.inventory === inventory || player.openInventory !== inventory) return@addListener
            playerSlots.restore()
            onClose()
        }
        sessionNode.addListener(PlayerAnvilInputEvent::class.java) { event ->
            if (event.inventory !== inventory) return@addListener
            onInput(event.input)
        }
        sessionNode.addListener(PlayerPacketEvent::class.java) { event ->
            val packet = event.packet
            if (packet is ClientPongPacket) onPong(packet.id())
        }
        MinecraftServer.getGlobalEventHandler().addChild(sessionNode)
        node = sessionNode
    }

    override fun close() {
        teardownListeners()
        playerSlots.restore()
        player.closeInventory()
    }

    override fun teardownListeners() {
        node?.let { MinecraftServer.getGlobalEventHandler().removeChild(it) }
        node = null
    }

    private companion object {
        val LOGGER = LoggerFactory.getLogger(LiveInventoryHandle::class.java)
        val NODE_ID = AtomicLong()
    }
}

package com.chunkzero.window.minestom

import com.chunkzero.window.host.HudChannel
import com.chunkzero.window.host.testkit.ClientUpdate
import com.chunkzero.window.host.testkit.HostFixture
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.entity.Player
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.EventFilter
import net.minestom.server.event.EventNode
import net.minestom.server.event.inventory.InventoryPreClickEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.item.ItemStack
import net.minestom.server.listener.AnvilListener
import net.minestom.server.listener.WindowListener
import net.minestom.server.network.ConnectionState
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.ActionBarPacket
import net.minestom.server.network.packet.server.play.BossBarPacket
import net.minestom.server.network.packet.server.play.BundlePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.ScoreboardObjectivePacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.network.player.GameProfile
import net.minestom.server.network.player.PlayerConnection
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.util.UUID

/** A headless Minestom player whose connection records what a client would show. */
internal class MinestomFixture : HostFixture<ItemStack> {
    private val connection = RecordingConnection()
    val player = Player(connection, GameProfile(UUID.randomUUID(), "WindowTest"))

    override val host: MinestomHost = MinestomHost.of(player)
    override val updates: List<ClientUpdate<ItemStack>>
        get() = connection.updates

    override fun click(
        windowSlot: Int,
        shift: Boolean,
        right: Boolean,
    ) {
        val packet =
            ClientClickWindowPacket(
                screen().windowId.toInt(),
                0,
                windowSlot.toShort(),
                (if (right) 1 else 0).toByte(),
                if (shift) ClientClickWindowPacket.ClickType.QUICK_MOVE else ClientClickWindowPacket.ClickType.PICKUP,
                emptyMap(),
                ItemStack.Hash.AIR,
            )
        WindowListener.clickWindowListener(packet, player)
    }

    override fun closeScreen() {
        WindowListener.closeWindowListener(ClientCloseWindowPacket(screen().windowId.toInt()), player)
        connection.closeScreen()
    }

    override fun typeInAnvil(text: String) = AnvilListener.nameItemListener(ClientNameItemPacket(text), player)

    override fun pong(id: Int) = EventDispatcher.call(PlayerPacketEvent(player, ClientPongPacket(id)))

    override fun tick() = player.scheduler().processTick()

    override fun guardPlayerInventory() {
        val guard = EventNode.type("player-inventory-guard", EventFilter.PLAYER).setPriority(-100)
        guard.addListener(InventoryPreClickEvent::class.java) { event ->
            if (event.inventory === player.inventory) event.isCancelled = true
        }
        player.eventNode().addChild(guard)
    }

    override fun setPlayerItem(
        slot: Int,
        item: ItemStack?,
    ) = player.inventory.setItemStack(slot, item ?: ItemStack.AIR)

    override fun playerItem(slot: Int): ItemStack? = player.inventory.getItemStack(slot).takeUnless { it.isAir }

    private fun screen() = checkNotNull(player.openInventory) { "No screen is open" }

    private companion object {
        init {
            MinecraftServer.init()
        }
    }
}

private class RecordingConnection : PlayerConnection() {
    val updates = mutableListOf<ClientUpdate<ItemStack>>()
    private var windowId: Int? = null

    fun closeScreen() {
        windowId = null
        updates += ClientUpdate.CloseScreen
    }

    override fun sendPacket(packet: SendablePacket) {
        when (val sent = SendablePacket.extractServerPacket(ConnectionState.PLAY, packet)) {
            is OpenWindowPacket -> {
                windowId = sent.windowId()
                updates += ClientUpdate.OpenScreen(sent.title())
            }

            is CloseWindowPacket -> {
                if (sent.windowId() == windowId) closeScreen()
            }

            is WindowItemsPacket -> {
                if (sent.windowId() == windowId) {
                    sent.items().forEachIndexed { slot, item -> updates += ClientUpdate.SetSlot(slot, item.visible()) }
                }
            }

            is SetSlotPacket -> {
                if (sent.windowId() ==
                    windowId
                ) {
                    updates += ClientUpdate.SetSlot(sent.slot().toInt(), sent.itemStack().visible())
                }
            }

            is BundlePacket -> {
                updates += ClientUpdate.BundleDelimiter
            }

            is PingPacket -> {
                updates += ClientUpdate.Ping(sent.id())
            }

            is ActionBarPacket -> {
                updates += ClientUpdate.Hud(HudChannel.ACTION_BAR, sent.text().takeUnless { it == Component.empty() })
            }

            is BossBarPacket -> {
                when (val action = sent.action()) {
                    is BossBarPacket.AddAction -> {
                        updates += ClientUpdate.Hud(HudChannel.BOSS_BAR, action.title())
                    }

                    is BossBarPacket.UpdateTitleAction -> {
                        updates +=
                            ClientUpdate.Hud(HudChannel.BOSS_BAR, action.title())
                    }

                    is BossBarPacket.RemoveAction -> {
                        updates += ClientUpdate.Hud(HudChannel.BOSS_BAR, null)
                    }

                    else -> {
                        Unit
                    }
                }
            }

            is ScoreboardObjectivePacket -> {
                updates +=
                    ClientUpdate.Hud(HudChannel.SIDEBAR, sent.objectiveValue().takeUnless { sent.mode() == DESTROY })
            }

            else -> {
                Unit
            }
        }
    }

    override fun getRemoteAddress(): SocketAddress = InetSocketAddress("127.0.0.1", 25565)

    private fun ItemStack.visible(): ItemStack? = takeUnless { it.isAir }

    private companion object {
        const val DESTROY: Byte = 1
    }
}

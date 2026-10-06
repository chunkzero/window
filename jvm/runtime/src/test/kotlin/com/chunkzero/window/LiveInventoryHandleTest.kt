package com.chunkzero.window

import com.chunkzero.window.internal.LiveInventoryHandle
import com.chunkzero.window.manifest.ButtonTooltip
import com.chunkzero.window.minestom.MinestomPlatform
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.entity.Player
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.EventFilter
import net.minestom.server.event.EventNode
import net.minestom.server.event.inventory.InventoryPreClickEvent
import net.minestom.server.inventory.InventoryType
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.listener.WindowListener
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.player.GameProfile
import net.minestom.server.network.player.PlayerConnection
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.util.UUID
import net.minestom.server.inventory.click.Click as MinestomClick

class LiveInventoryHandleTest :
    StringSpec({
        "open container packet clicks in the player region route to player slots" {
            val player = testPlayer()
            val clicks = mutableListOf<Click>()
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x3",
                    buttons =
                        mapOf(
                            "home" to
                                TestManifests.buttonRefs((9..17).map(TestManifests::playerSlot)),
                        ),
                )
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("home") { clicks += it }
                    }
                }

            val session = Windows.load(manifest, MinestomPlatform).open(player, view)
            try {
                val packet =
                    ClientClickWindowPacket(
                        session.inventory.windowId.toInt(),
                        0,
                        session.inventory.size.toShort(),
                        0,
                        ClientClickWindowPacket.ClickType.PICKUP,
                        emptyMap<Short, ItemStack.Hash>(),
                        ItemStack.Hash.AIR,
                    )
                val processed =
                    player.clickPreprocessor.processClick(packet, session.inventory.size)
                val windowClick = MinestomClick.toWindow(processed!!, session.inventory.size)
                windowClick.inOpened() shouldBe false
                windowClick.click().slot() shouldBe 9

                EventDispatcher.call(
                    InventoryPreClickEvent(player.inventory, player, windowClick.click()),
                )

                clicks shouldHaveSize 1
                clicks.single().slot shouldBe SlotRef(SlotArea.PLAYER, 9)
            } finally {
                session.close()
            }
        }

        "window listener routes visible player inventory rows to player slots" {
            val player = testPlayer()
            val clicks = mutableListOf<Click>()
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x3",
                    buttons =
                        mapOf(
                            "home" to
                                TestManifests.buttonRefs((9..17).map(TestManifests::playerSlot)),
                            "help" to
                                TestManifests.buttonRefs((18..26).map(TestManifests::playerSlot)),
                        ),
                )
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("home") { clicks += it }
                        button("help") { clicks += it }
                    }
                }

            val session = Windows.load(manifest, MinestomPlatform).open(player, view)
            try {
                WindowListener.clickWindowListener(
                    clickPacket(session, session.inventory.size),
                    player,
                )
                WindowListener.clickWindowListener(
                    clickPacket(session, session.inventory.size + 9),
                    player,
                )

                clicks.map { it.slot } shouldBe
                    listOf(SlotRef(SlotArea.PLAYER, 9), SlotRef(SlotArea.PLAYER, 18))
            } finally {
                session.close()
            }
        }

        "window listener routes visible hotbar cells to player hotbar slots" {
            val player = testPlayer()
            val clicks = mutableListOf<Click>()
            val manifest =
                TestManifests.manifest(
                    container = "anvil",
                    buttons =
                        mapOf(
                            "left" to TestManifests.buttonRefs(listOf(TestManifests.playerSlot(0))),
                            "middle" to
                                TestManifests.buttonRefs(listOf(TestManifests.playerSlot(4))),
                            "right" to TestManifests.buttonRefs(listOf(TestManifests.playerSlot(8))),
                        ),
                )
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("left") { clicks += it }
                        button("middle") { clicks += it }
                        button("right") { clicks += it }
                    }
                }

            val session = Windows.load(manifest, MinestomPlatform).open(player, view)
            try {
                for (hotbarSlot in listOf(0, 4, 8)) {
                    WindowListener.clickWindowListener(
                        clickPacket(session, session.inventory.size + 27 + hotbarSlot),
                        player,
                    )
                }

                clicks.map { it.slot } shouldBe
                    listOf(
                        SlotRef(SlotArea.PLAYER, 0),
                        SlotRef(SlotArea.PLAYER, 4),
                        SlotRef(SlotArea.PLAYER, 8),
                    )
            } finally {
                session.close()
            }
        }

        "player inventory clicks route even after an earlier guard cancels them" {
            val player = testPlayer()
            val clicks = mutableListOf<Click>()
            val guardNode =
                EventNode
                    .event(
                        "pre-cancelling-player-inventory-guard",
                        EventFilter.INVENTORY,
                        { event -> event.inventory === player.inventory },
                    ).setPriority(-100)
            guardNode.addListener(InventoryPreClickEvent::class.java) { event ->
                event.isCancelled = true
            }
            MinecraftServer.getGlobalEventHandler().addChild(guardNode)

            val manifest =
                TestManifests.manifest(
                    container = "generic_9x3",
                    buttons =
                        mapOf(
                            "home" to
                                TestManifests.buttonRefs((9..17).map(TestManifests::playerSlot)),
                        ),
                )
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("home") { clicks += it }
                    }
                }

            val session = Windows.load(manifest, MinestomPlatform).open(player, view)
            try {
                WindowListener.clickWindowListener(
                    clickPacket(session, session.inventory.size),
                    player,
                )

                clicks.single().slot shouldBe SlotRef(SlotArea.PLAYER, 9)
            } finally {
                session.close()
                MinecraftServer.getGlobalEventHandler().removeChild(guardNode)
            }
        }

        "player slot windows clear and restore the visible player inventory" {
            val connection = RecordingConnection()
            val player = testPlayer(connection)
            player.inventory.setItemStack(0, ItemStack.of(Material.DIAMOND))
            player.inventory.setItemStack(8, ItemStack.of(Material.EMERALD))
            player.inventory.setItemStack(9, ItemStack.of(Material.GOLD_INGOT))
            player.inventory.setItemStack(35, ItemStack.of(Material.IRON_INGOT))
            val manifest =
                TestManifests.manifest(
                    container = "generic_9x3",
                    buttons =
                        mapOf(
                            "home" to
                                TestManifests.buttonRefs(
                                    listOf(
                                        TestManifests.playerSlot(9),
                                        TestManifests.playerSlot(0),
                                    ),
                                    tooltip = ButtonTooltip("Home"),
                                ),
                        ),
                )
            val view =
                object : WindowView("w") {
                    override fun WindowScope.bind() {
                        button("home") {}
                    }
                }

            val session = Windows.load(manifest, MinestomPlatform).open(player, view)
            try {
                player.inventory.getItemStack(8) shouldBe ItemStack.AIR
                player.inventory.getItemStack(35) shouldBe ItemStack.AIR
                player.inventory.getItemStack(9).material() shouldBe Material.PAPER

                connection.packets
                    .filterIsInstance<SetSlotPacket>()
                    .filter { it.windowId() == session.inventory.windowId.toInt() }
                    .map { it.slot().toInt() }
                    .containsAll(listOf(27, 54)) shouldBe true
            } finally {
                session.close()
            }
            player.inventory.getItemStack(0).material() shouldBe Material.DIAMOND
            player.inventory.getItemStack(8).material() shouldBe Material.EMERALD
            player.inventory.getItemStack(9).material() shouldBe Material.GOLD_INGOT
            player.inventory.getItemStack(35).material() shouldBe Material.IRON_INGOT
        }

        "opening another window from a click retires the previous session" {
            val player = testPlayer()
            player.inventory.setItemStack(0, ItemStack.of(Material.DIAMOND))
            val manifest =
                TestManifests.manifest(
                    buttons =
                        mapOf(
                            "home" to
                                TestManifests.buttonRefs(
                                    listOf(TestManifests.playerSlot(0)),
                                    tooltip = ButtonTooltip("Home"),
                                ),
                        ),
                )
            val windows = Windows.load(manifest, MinestomPlatform)
            val events = mutableListOf<String>()
            lateinit var next: WindowSession

            class Recording(
                val label: String,
                val onHome: () -> Unit,
            ) : WindowView("w") {
                override fun WindowScope.bind() {
                    button("home") {
                        events += "$label:click"
                        onHome()
                    }
                }

                override fun onClose() {
                    events += "$label:close"
                }
            }

            val first =
                windows.open(
                    player,
                    Recording("first") { next = windows.open(player, Recording("second") {}) },
                )
            WindowListener.clickWindowListener(clickPacket(first, first.inventory.size + 27), player)
            WindowListener.clickWindowListener(clickPacket(next, next.inventory.size + 27), player)
            next.close()

            events shouldBe listOf("first:click", "first:close", "second:click", "second:close")
            player.inventory.getItemStack(0).material() shouldBe Material.DIAMOND
        }

        "opened-inventory player-region pre-clicks are normalised by container size" {
            val player = testPlayer()
            val handle = LiveInventoryHandle(player, InventoryType.CHEST_3_ROW, MinestomPlatform)
            val clicks = mutableListOf<com.chunkzero.window.internal.ClickInfo>()
            handle.open(Component.text("test"))
            handle.registerListeners({ clicks += it }, {}, {}, {})
            try {
                EventDispatcher.call(
                    InventoryPreClickEvent(
                        handle.inventory,
                        player,
                        MinestomClick.Left(handle.inventory.size + 18),
                    ),
                )

                clicks.single().slot shouldBe SlotRef(SlotArea.PLAYER, 18)
            } finally {
                handle.close()
            }
        }

        "anvil input seeds cannot hold enchantments, so the client never prices a repair" {
            WindowItems.anvilInput("maps").has(DataComponents.ENCHANTMENTS) shouldBe false
        }
    })

private fun clickPacket(
    session: WindowSession,
    slot: Int,
): ClientClickWindowPacket =
    ClientClickWindowPacket(
        session.inventory.windowId.toInt(),
        0,
        slot.toShort(),
        0,
        ClientClickWindowPacket.ClickType.PICKUP,
        emptyMap<Short, ItemStack.Hash>(),
        ItemStack.Hash.AIR,
    )

private fun testPlayer(): Player = testPlayer(RecordingConnection())

private fun testPlayer(connection: RecordingConnection): Player {
    MinecraftServer.init()
    return Player(connection, GameProfile(UUID.randomUUID(), "WindowTest"))
}

private class RecordingConnection : PlayerConnection() {
    val packets = mutableListOf<SendablePacket>()

    override fun sendPacket(packet: SendablePacket) {
        packets += packet
    }

    override fun getRemoteAddress(): SocketAddress = InetSocketAddress("127.0.0.1", 25565)
}

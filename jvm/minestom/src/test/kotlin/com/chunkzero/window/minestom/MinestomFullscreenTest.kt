package com.chunkzero.window.minestom

import com.chunkzero.window.FullscreenClick
import com.chunkzero.window.FullscreenDefinition
import com.chunkzero.window.FullscreenScope
import com.chunkzero.window.FullscreenView
import com.chunkzero.window.manifest.ControlEntry
import com.chunkzero.window.manifest.CursorEntry
import com.chunkzero.window.manifest.FullscreenCanvasEntry
import com.chunkzero.window.manifest.FullscreenEntry
import com.chunkzero.window.manifest.FullscreenMode
import com.chunkzero.window.manifest.FullscreenSurfaceEntry
import com.chunkzero.window.manifest.HorseEntry
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainAll
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.entity.Entity
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.EquipmentSlot
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.player.PlayerPacketOutEvent
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.ConnectionState
import net.minestom.server.network.packet.client.ClientPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.play.CameraPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityEquipmentPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.SetPassengersPacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.network.player.GameProfile
import net.minestom.server.network.player.PlayerConnection
import java.net.InetSocketAddress
import java.net.SocketAddress
import java.util.UUID

class MinestomFullscreenTest :
    StringSpec({
        "opening pins the player in spectator mode, fakes survival next tick, and closing restores them" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val start = Pos(4.0, 42.0, 4.0, 90f, 10f)
            player.teleport(start).join()
            player.setHeldItemSlot(3)
            player.isInvulnerable = true
            player.isInstantBreak = true
            val session = Menu(player).open()

            player.gameMode shouldBe GameMode.SPECTATOR
            player.position shouldBe start.withView(0f, 0f)
            player.flyingSpeed shouldBe 0f
            fixture.sent<SpawnEntityPacket>().map { it.type() } shouldContainExactly
                listOf(EntityType.TEXT_DISPLAY, EntityType.INTERACTION)
            val hotbar = fixture.sent<SetPlayerInventorySlotPacket>()
            hotbar.map { it.slot() } shouldContainExactly (0..8).toList()
            hotbar.first().itemStack().get(DataComponents.ITEM_MODEL) shouldBe "window:fullscreen/blank"
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
            fixture
                .sent<EntityAttributesPacket>()
                .single()
                .properties()
                .single()
                .value() shouldBe 20.0
            fixture.sent<ChangeGameStatePacket>().map { it.value() } shouldBe
                listOf(GameMode.SPECTATOR.ordinal.toFloat(), 0f)

            fixture.clear()
            fixture.tick()
            fixture.sent<ChangeGameStatePacket>().single().value() shouldBe 0f
            fixture
                .sent<PlayerInfoUpdatePacket>()
                .single()
                .entries()
                .single()
                .gameMode() shouldBe GameMode.SURVIVAL
            fixture.sent<PlayerAbilitiesPacket>().single().flyingSpeed() shouldBe 0f

            fixture.clear()
            session.close()
            player.gameMode shouldBe GameMode.SURVIVAL
            player.position shouldBe start
            player.heldSlot shouldBe 3.toByte()
            player.flyingSpeed shouldBe 0.05f
            player.isInvulnerable shouldBe true
            player.isInstantBreak shouldBe true
            fixture
                .sent<DestroyEntitiesPacket>()
                .single()
                .entityIds()
                .size shouldBe 2
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe player.instance.worldAge
            fixture
                .sent<EntityAttributesPacket>()
                .single()
                .properties()
                .single()
                .value() shouldBe 40.0
        }

        "a click is hit-tested at the end of the player's tick, after the rotation the client sent with it" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()

            fixture.receive(ClientAttackPacket(0))
            // Canvas pixel (5, 5): the button's top-left corner.
            fixture.receive(ClientPlayerRotationPacket(-44f, -24.375f, 0.toByte()))
            menu.clicks shouldBe emptyList()
            fixture.player.scheduler().processTickEnd()

            menu.clicks shouldContainExactly listOf(FullscreenClick(5, 5))
        }

        "sneaking leaves the window and restores the player" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()

            fixture.receive(ClientInputPacket(false, false, false, false, false, true, false))

            menu.closed shouldBe true
            fixture.player.gameMode shouldBe GameMode.SURVIVAL
        }

        "a second full-screen window cannot open over the first" {
            val fixture = FullscreenFixture()
            Menu(fixture.player).open()

            shouldThrow<IllegalStateException> { Menu(fixture.player).open() }
        }

        "time, max health, and absorption are followed by the hidden value, and a container gets the real ones" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.inventory.setItemStack(2, ItemStack.of(Material.DIAMOND))
            Menu(player).open()
            fixture.clear()

            fixture.send(SetTimePacket(100, emptyMap()))
            fixture.sent<SetTimePacket>().map { it.gameTime() } shouldContainExactly listOf(100L, HIDDEN_AGE)
            fixture.clear()
            val clock =
                player.instance
                    .createTimePacket()
                    .clocks()
                    .keys
                    .first()
            val daylight = mapOf(clock to SetTimePacket.ClockState(6000, 0f, 1f))
            fixture.send(SetTimePacket(10_000, daylight))
            fixture.sent<SetTimePacket>().map { it.clocks() } shouldContainExactly listOf(daylight, daylight)
            fixture.clear()
            val uncapped = EntityAttributesPacket.Property(Attribute.MAX_HEALTH, 40.0, emptyList())
            fixture.send(EntityAttributesPacket(player.entityId, listOf(uncapped)))
            fixture
                .sent<EntityAttributesPacket>()
                .map { it.properties().single().value() } shouldContainExactly listOf(40.0, 20.0)
            fixture.clear()
            val absorption = MetadataDef.Player.ADDITIONAL_HEARTS.index()
            fixture.send(EntityMetaDataPacket(player.entityId, mapOf(absorption to Metadata.Float(4f))))
            fixture
                .sent<EntityMetaDataPacket>()
                .map { it.entries()[absorption]?.value() } shouldContainExactly listOf(4f, 0f)

            fixture.clear()
            fixture.send(OpenWindowPacket(1, InventoryType.CHEST_1_ROW.ordinal, Component.empty()))
            fixture.sent<SetTimePacket>().single() shouldBe player.instance.createTimePacket()
            // Opening a container leaves the hotbar blank unless the real one is resent.
            fixture
                .sent<WindowItemsPacket>()
                .single()
                .items()[38]
                .material() shouldBe Material.DIAMOND
            fixture.clear()
            fixture.send(SetTimePacket(100, emptyMap()))
            fixture.sent<SetTimePacket>().map { it.gameTime() } shouldContainExactly listOf(100L)
        }

        "game mode, abilities, and hotbar updates are followed by hidden state, and a changed mode survives" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val session = Menu(player).open()
            fixture.clear()

            fixture.send(ChangeGameStatePacket(ChangeGameStatePacket.Reason.CHANGE_GAMEMODE, 1f))
            fixture.sent<ChangeGameStatePacket>().map { it.value() } shouldContainExactly listOf(1f, 0f)
            fixture.clear()
            val creative =
                PlayerInfoUpdatePacket.Entry(
                    player.uuid,
                    "",
                    emptyList(),
                    false,
                    0,
                    GameMode.CREATIVE,
                    null,
                    null,
                    0,
                    false,
                )
            fixture.send(PlayerInfoUpdatePacket(PlayerInfoUpdatePacket.Action.UPDATE_GAME_MODE, creative))
            fixture
                .sent<PlayerInfoUpdatePacket>()
                .map { it.entries().single().gameMode() } shouldContainExactly
                listOf(GameMode.CREATIVE, GameMode.SURVIVAL)
            fixture.clear()
            fixture.send(PlayerAbilitiesPacket(0x0F, 0.05f, 0.1f))
            fixture.sent<PlayerAbilitiesPacket>().map { it.flyingSpeed() } shouldContainExactly listOf(0.05f, 0f)

            fixture.clear()
            val diamond = ItemStack.of(Material.DIAMOND)
            fixture.send(SetPlayerInventorySlotPacket(2, diamond))
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
            fixture.clear()
            fixture.send(SetSlotPacket(0, 1, 38, diamond))
            fixture.hotbar() shouldBe List(9) { Material.PAPER }

            player.setGameMode(GameMode.CREATIVE)
            session.close()
            player.gameMode shouldBe GameMode.CREATIVE
            fixture.clear()
            fixture.send(SetTimePacket(100, emptyMap()))
            fixture.send(SetPlayerInventorySlotPacket(2, diamond))
            fixture.sent<SetTimePacket>().map { it.gameTime() } shouldContainExactly listOf(100L)
            fixture.sent<SetPlayerInventorySlotPacket>().map { it.itemStack() } shouldContainExactly listOf(diamond)
        }

        "rewrites keep the order of queued packets, so an old inventory snapshot cannot overwrite a newer slot" {
            val fixture = FullscreenFixture()
            Menu(fixture.player).open()
            fixture.clear()
            val diamond = ItemStack.of(Material.DIAMOND)
            val stick = ItemStack.of(Material.STICK)

            fixture.send(
                WindowItemsPacket(0, 1, List(46) { diamond }, ItemStack.AIR),
                SetSlotPacket(0, 2, 38, ItemStack.of(Material.EMERALD)),
                SetSlotPacket(-1, 2, -1, stick),
            )

            val delivered = fixture.delivered
            delivered.take(3).map { it::class } shouldContainExactly
                listOf(WindowItemsPacket::class, SetSlotPacket::class, SetSlotPacket::class)
            (delivered[0] as WindowItemsPacket).items()[10] shouldBe diamond
            (delivered[1] as SetSlotPacket).itemStack().material() shouldBe Material.EMERALD
            (delivered[2] as SetSlotPacket).itemStack() shouldBe stick
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
        }

        "a container close for a window that is not open does not hide the newer container" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.openInventory(Inventory(InventoryType.CHEST_1_ROW, Component.empty()))
            val first = player.openInventory!!.windowId.toInt()
            val second = first + 1
            fixture.clear()
            Menu(player).open()
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.sent<SetPlayerInventorySlotPacket>() shouldBe emptyList()

            fixture.send(CloseWindowPacket(first))
            fixture.send(OpenWindowPacket(second, InventoryType.CHEST_1_ROW.ordinal, Component.empty()))
            fixture.clear()
            fixture.tick()
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.sent<SetPlayerInventorySlotPacket>() shouldBe emptyList()

            // Reopening the same inventory reuses its window id, so the earlier close must not act next tick.
            fixture.send(CloseWindowPacket(second))
            fixture.send(OpenWindowPacket(second, InventoryType.CHEST_1_ROW.ordinal, Component.empty()))
            fixture.clear()
            fixture.tick()
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.sent<SetPlayerInventorySlotPacket>() shouldBe emptyList()

            fixture.receive(ClientCloseWindowPacket(first))
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.receive(ClientCloseWindowPacket(second))
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
        }

        "horse mode locks the view next tick, leaves max health alone, and unlocks it on close" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val session = Menu(player, FullscreenMode.HORSE).open()

            fixture.sent<SpawnEntityPacket>().map { it.type() } shouldContainExactly
                listOf(EntityType.TEXT_DISPLAY, EntityType.INTERACTION)
            fixture.sent<EntityAttributesPacket>() shouldBe emptyList()
            fixture.clear()
            fixture.tick()

            val spawns = fixture.sent<SpawnEntityPacket>()
            spawns.map { it.type() } shouldContainExactly
                listOf(EntityType.BAT, EntityType.ITEM_DISPLAY, EntityType.HORSE)
            val (bat, seat, horse) = spawns
            bat.position().y() shouldBe player.position.y() + player.eyeHeight - Entity(EntityType.BAT).eyeHeight
            horse.position() shouldBe player.position.add(0.0, player.eyeHeight - 1.75, 3.0)
            seat.position() shouldBe horse.position()
            val armor =
                fixture
                    .sent<EntityEquipmentPacket>()
                    .single()
                    .equipments()
                    .getValue(EquipmentSlot.BODY)
            armor.get(DataComponents.EQUIPPABLE)?.assetId() shouldBe "window:fullscreen/horse_cursor"
            fixture.sent<SetPassengersPacket>() shouldContainExactly
                listOf(
                    SetPassengersPacket(seat.entityId(), listOf(horse.entityId())),
                    SetPassengersPacket(horse.entityId(), listOf(player.entityId)),
                )
            fixture.sent<CameraPacket>().single().cameraId() shouldBe bat.entityId()

            fixture.clear()
            fixture.send(
                EntityAttributesPacket(
                    player.entityId,
                    listOf(EntityAttributesPacket.Property(Attribute.MAX_HEALTH, 40.0, emptyList())),
                ),
            )
            fixture.sent<EntityAttributesPacket>().size shouldBe 1

            fixture.clear()
            session.close()
            fixture.sent<CameraPacket>().single().cameraId() shouldBe player.entityId
            fixture.sent<SetPassengersPacket>().single().passengersId() shouldBe emptyList()
            fixture.sent<DestroyEntitiesPacket>().flatMap { it.entityIds() } shouldContainAll
                listOf(bat.entityId(), horse.entityId(), seat.entityId())
        }
    }) {
    private companion object {
        const val HIDDEN_AGE = -1_000_000_001L
    }
}

private class Menu(
    player: Player,
    mode: FullscreenMode = FullscreenMode.UNLOCKED,
) : FullscreenView(FullscreenDefinition(manifest(mode), "menu"), MinestomFullscreenHost(player)) {
    val clicks = mutableListOf<FullscreenClick>()
    var closed = false

    override fun FullscreenScope.bind() {
        button("press") { clicks += it }
    }

    override fun onClose() {
        closed = true
    }
}

/** A player in an instance, with 40 max health, whose connection records every packet that reaches its client. */
private class FullscreenFixture {
    private val connection = PacketLog()
    val player = Player(connection, GameProfile(UUID.randomUUID(), "Fullscreen"))

    init {
        val instance = MinecraftServer.getInstanceManager().createInstanceContainer()
        player.setInstance(instance, Pos(0.0, 42.0, 0.0)).join()
        player.getAttribute(Attribute.MAX_HEALTH).baseValue = 40.0
        clear()
    }

    val delivered: List<ServerPacket>
        get() = connection.delivered

    inline fun <reified T : ServerPacket> sent(): List<T> {
        flush()
        return delivered.filterIsInstance<T>()
    }

    /** The materials the client shows in its hotbar, replaying every packet delivered since the player spawned. */
    fun hotbar(): List<Material> {
        flush()
        return connection.hotbar.toList()
    }

    fun clear() {
        flush()
        connection.delivered.clear()
    }

    fun tick() = player.scheduler().processTick()

    fun receive(packet: ClientPacket) = EventDispatcher.call(PlayerPacketEvent(player, packet))

    /** Queues [packets] back to back, then writes them out the way the connection's writer thread does. */
    fun send(vararg packets: ServerPacket) {
        packets.forEach(connection::sendPacket)
        flush()
    }

    private fun flush() = connection.flush(player)

    private companion object {
        init {
            MinecraftServer.init()
        }
    }
}

/** Queues packets and writes them in order, dispatching the outgoing packet event for each as a socket connection does. */
private class PacketLog : PlayerConnection() {
    private val queue = ArrayDeque<ServerPacket>()
    val delivered = mutableListOf<ServerPacket>()
    val hotbar = MutableList(9) { Material.AIR }

    override fun sendPacket(packet: SendablePacket) {
        queue += SendablePacket.extractServerPacket(ConnectionState.PLAY, packet) ?: return
    }

    fun flush(player: Player) {
        while (queue.isNotEmpty()) {
            val packet = queue.removeFirst()
            val event = PlayerPacketOutEvent(player, packet)
            EventDispatcher.call(event)
            if (event.isCancelled) continue
            delivered += packet
            when (packet) {
                is WindowItemsPacket -> {
                    if (packet.windowId() ==
                        0
                    ) {
                        hotbar.indices.forEach { hotbar[it] = packet.items()[36 + it].material() }
                    }
                }

                is SetSlotPacket -> {
                    if (packet.windowId() == 0 &&
                        packet.slot() in 36..44
                    ) {
                        hotbar[packet.slot() - 36] = packet.itemStack().material()
                    }
                }

                is SetPlayerInventorySlotPacket -> {
                    if (packet.slot() in hotbar.indices) hotbar[packet.slot()] = packet.itemStack().material()
                }

                else -> {
                    Unit
                }
            }
        }
    }

    override fun getRemoteAddress(): SocketAddress = InetSocketAddress("127.0.0.1", 25565)
}

private fun manifest(mode: FullscreenMode): WindowManifest {
    val magnitudes = (0..10).map { 1 shl it }
    val advances = magnitudes.reversed().map { -it } + magnitudes
    val canvas =
        FullscreenCanvasEntry(
            width = 480,
            height = 270,
            degreesPerPixel = 0.1875,
            edgeYaw = 45.0,
            edgePitch = 25.3125,
            snapEpsilon = 0.01,
            hitMargin = 1,
            displayScale = 256.0,
            markerAlpha = 253,
            cursor = CursorEntry(text = "C", marker = "#a7fc00", width = 8, height = 12),
            backdrop = "B",
            overlayMarker = "#a3f800",
            hudHiddenWorldAge = -1_000_000_001,
            hudStripHeight = 56,
            hudStripHalfWidth = 96,
            hudMaxHealth = 20,
            horse = HorseEntry(listOf(0.0, -1.75, 3.0), "window:fullscreen/horse_cursor"),
        )
    return WindowManifest(
        version = WindowManifest.VERSION,
        namespace = "window",
        font = "window:ui",
        spacers = advances.withIndex().associate { (index, advance) -> 0xF0000 + index to advance },
        textAdvances = ('A'..'Z').associate { it.toString() to 6 },
        windows = emptyMap(),
        fullscreens =
            mapOf(
                "menu" to
                    FullscreenEntry(
                        surface = FullscreenSurfaceEntry("fullscreen", mode, listOf(480, 270)),
                        static = "S",
                        staticMarker = "#a00400",
                        controls = mapOf("press" to ControlEntry(0, 5, 5, 20, 10, action = "press", hover = false)),
                    ),
            ),
        fullscreenCanvas = canvas,
    )
}

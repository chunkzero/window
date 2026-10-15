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
import com.chunkzero.window.manifest.WindowManifest
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.shouldBe
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.player.PlayerPacketOutEvent
import net.minestom.server.inventory.InventoryType
import net.minestom.server.network.ConnectionState
import net.minestom.server.network.packet.client.ClientPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
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
                listOf(GameMode.SPECTATOR.ordinal.toFloat())

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

        "a click is hit-tested at the end of the tick, after the rotation the client sent with it" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()

            fixture.receive(ClientAttackPacket(0))
            // Canvas pixel (5, 5): the button's top-left corner.
            fixture.receive(ClientPlayerRotationPacket(-44f, -24.375f, 0.toByte()))
            menu.clicks shouldBe emptyList()
            MinecraftServer.getSchedulerManager().processTickEnd()

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

        "outgoing time, max health, and absorption keep the HUD hidden, and a container gets the real time" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            Menu(player).open()
            fixture.clear()

            fixture.sendOut(SetTimePacket(100, emptyMap())) shouldBe false
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
            val uncapped = EntityAttributesPacket.Property(Attribute.MAX_HEALTH, 40.0, emptyList())
            fixture.sendOut(EntityAttributesPacket(player.entityId, listOf(uncapped))) shouldBe false
            val capped = fixture.sent<EntityAttributesPacket>().single()
            capped.properties().single().value() shouldBe 20.0
            fixture.sendOut(capped) shouldBe true
            val absorption = MetadataDef.Player.ADDITIONAL_HEARTS.index()
            val hearts = EntityMetaDataPacket(player.entityId, mapOf(absorption to Metadata.Float(4f)))
            fixture.sendOut(hearts) shouldBe false
            fixture
                .sent<EntityMetaDataPacket>()
                .single()
                .entries()[absorption]
                ?.value() shouldBe 0f

            fixture.clear()
            fixture.sendOut(OpenWindowPacket(1, InventoryType.CHEST_1_ROW.ordinal, Component.empty())) shouldBe true
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe player.instance.worldAge
            fixture.sendOut(SetTimePacket(100, emptyMap())) shouldBe true
        }
    }) {
    private companion object {
        const val HIDDEN_AGE = -1_000_000_001L
    }
}

private class Menu(
    player: Player,
) : FullscreenView(FullscreenDefinition(manifest(), "menu"), MinestomFullscreenHost(player)) {
    val clicks = mutableListOf<FullscreenClick>()
    var closed = false

    override fun FullscreenScope.bind() {
        button("press") { clicks += it }
    }

    override fun onClose() {
        closed = true
    }
}

/** A player in an instance, with 40 max health, whose connection records every packet sent to it. */
private class FullscreenFixture {
    private val connection = PacketLog()
    val player = Player(connection, GameProfile(UUID.randomUUID(), "Fullscreen"))

    init {
        val instance = MinecraftServer.getInstanceManager().createInstanceContainer()
        player.setInstance(instance, Pos(0.0, 42.0, 0.0)).join()
        player.getAttribute(Attribute.MAX_HEALTH).baseValue = 40.0
        connection.packets.clear()
    }

    inline fun <reified T : ServerPacket> sent(): List<T> = connection.packets.filterIsInstance<T>()

    fun clear() = connection.packets.clear()

    fun tick() = player.scheduler().processTick()

    fun receive(packet: ClientPacket) = EventDispatcher.call(PlayerPacketEvent(player, packet))

    /** Sends [packet] through the outgoing packet event, as a real connection does; false when it was cancelled. */
    fun sendOut(packet: ServerPacket): Boolean {
        val event = PlayerPacketOutEvent(player, packet)
        EventDispatcher.call(event)
        return !event.isCancelled
    }

    private companion object {
        init {
            MinecraftServer.init()
        }
    }
}

private class PacketLog : PlayerConnection() {
    val packets = mutableListOf<ServerPacket>()

    override fun sendPacket(packet: SendablePacket) {
        packets += SendablePacket.extractServerPacket(ConnectionState.PLAY, packet) ?: return
    }

    override fun getRemoteAddress(): SocketAddress = InetSocketAddress("127.0.0.1", 25565)
}

private fun manifest(): WindowManifest {
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
                        surface = FullscreenSurfaceEntry("fullscreen", FullscreenMode.UNLOCKED, listOf(480, 270)),
                        static = "S",
                        staticMarker = "#a00400",
                        controls = mapOf("press" to ControlEntry(0, 5, 5, 20, 10, action = "press", hover = false)),
                    ),
            ),
        fullscreenCanvas = canvas,
    )
}

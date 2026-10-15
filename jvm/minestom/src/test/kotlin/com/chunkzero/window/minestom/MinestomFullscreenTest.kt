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
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.coordinate.Vec
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.PlayerHand
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
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientHeldItemChangePacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientInteractEntityPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.HeldItemChangePacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.SetExperiencePacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.network.packet.server.play.WindowPropertyPacket
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

        "held-slot changes are steps the shortest way around the hotbar, from the slot the client last had" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()

            fixture.receive(ClientHeldItemChangePacket(1)).isCancelled shouldBe true
            fixture.receive(ClientHeldItemChangePacket(8))
            fixture.send(HeldItemChangePacket(5))
            fixture.receive(ClientHeldItemChangePacket(6))

            menu.scrolls shouldContainExactly listOf(1, -2, 1)
            fixture.player.heldSlot shouldBe 0.toByte()
        }

        "a main-hand right click is reported at the end of the player's tick" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()

            fixture.receive(ClientInteractEntityPacket(0, PlayerHand.MAIN, Vec.ZERO, false)).isCancelled shouldBe true
            fixture.receive(ClientInteractEntityPacket(0, PlayerHand.OFF, Vec.ZERO, false))
            fixture.receive(ClientPlayerRotationPacket(-44f, -24.375f, 0.toByte()))
            fixture.player.scheduler().processTickEnd()

            menu.rightClicks shouldContainExactly listOf(FullscreenClick(5, 5))
        }

        "a text input opens a blank anvil, ignores renames until its ping returns, and submits from the result slot" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.inventory.setItemStack(2, ItemStack.of(Material.DIAMOND))
            val menu = Menu(player)
            menu.open()
            fixture.tick()
            fixture.clear()

            menu.prompt("hi")
            val anvil = fixture.sent<OpenWindowPacket>().single()
            anvil.windowType() shouldBe InventoryType.ANVIL.windowType
            // The HUD stays hidden, the hotbar stays blank, and the anvil shows no cost.
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
            fixture.seed(anvil.windowId()) shouldBe "hi"
            val paper = fixture.seedItem(anvil.windowId())
            paper.get(DataComponents.ITEM_MODEL) shouldBe "window:fullscreen/blank"
            paper.has(DataComponents.ENCHANTMENTS) shouldBe false
            fixture.sent<WindowPropertyPacket>().single().value() shouldBe 0
            fixture.receive(ClientNameItemPacket("earlier"))
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))
            fixture.receive(ClientNameItemPacket("hi"))
            fixture.receive(ClientNameItemPacket("hey"))
            menu.edits shouldContainExactly listOf("hey")

            // A full field reopens the same anvil with an empty one.
            val full = "a".repeat(50)
            fixture.clear()
            fixture.receive(ClientNameItemPacket(full))
            fixture.sent<OpenWindowPacket>().single().windowId() shouldBe anvil.windowId()
            fixture.seed(anvil.windowId()) shouldBe ""
            fixture.receive(ClientNameItemPacket(full.drop(1)))
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))
            fixture.receive(ClientNameItemPacket("b"))
            menu.edits shouldContainExactly listOf("hey", full, full + "b")

            fixture.clear()
            fixture.receive(fixture.click(anvil.windowId(), 0))
            fixture.seed(anvil.windowId()) shouldBe "b"
            fixture.receive(fixture.click(anvil.windowId(), 2))
            menu.prompts shouldContainExactly listOf(full + "b")
            player.openInventory shouldBe null
            fixture.clear()
            fixture.tick()
            fixture.sent<SetTimePacket>() shouldBe emptyList()
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
        }

        "a click that leaves the input slot alone resends the slots without a ping, and later renames are kept" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.prompt("hey")
            val anvil = fixture.sent<OpenWindowPacket>().single().windowId()
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))

            fixture.clear()
            fixture.receive(fixture.click(anvil, 1))
            fixture.sent<PingPacket>() shouldBe emptyList()
            fixture.seed(anvil) shouldBe "hey"
            fixture.sent<WindowPropertyPacket>().single().value() shouldBe 0

            // Text typed while the resync was in flight is reported, and a submit sees it.
            fixture.receive(ClientNameItemPacket("heya"))
            menu.edits shouldContainExactly listOf("heya")
            fixture.receive(fixture.click(anvil, 2))
            menu.prompts shouldContainExactly listOf("heya")
        }

        "a click that changes the input slot reseeds it behind a ping, and renames before the pong are dropped" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.prompt("hey")
            val anvil = fixture.sent<OpenWindowPacket>().single().windowId()
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))
            fixture.receive(ClientNameItemPacket("hey!"))

            fixture.clear()
            fixture.receive(fixture.click(anvil, 0))
            fixture.seed(anvil) shouldBe "hey!"
            fixture.receive(ClientNameItemPacket("stale"))
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))
            fixture.receive(ClientNameItemPacket("hey!!"))
            menu.edits shouldContainExactly listOf("hey!", "hey!!")

            // A shift-click that moved the input paper out reports it among the changed slots.
            fixture.clear()
            fixture.receive(fixture.click(anvil, 3, changed = mapOf(0.toShort() to ItemStack.Hash.AIR)))
            fixture.seed(anvil) shouldBe "hey!!"
            fixture.sent<PingPacket>().size shouldBe 1
        }

        "a click during a reopen does not admit the old field's renames into the new segment" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            val almost = "a".repeat(49)
            menu.prompt(almost)
            val anvil = fixture.sent<OpenWindowPacket>().single().windowId()
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))

            fixture.clear()
            fixture.receive(ClientNameItemPacket(almost + "a"))
            val reopen = fixture.sent<PingPacket>().single()
            fixture.clear()
            fixture.receive(fixture.click(anvil, 0))
            val reseed = fixture.sent<PingPacket>().single()
            fixture.receive(ClientNameItemPacket(almost))
            fixture.receive(ClientPongPacket(reopen.id()))
            fixture.receive(ClientNameItemPacket(almost))
            fixture.receive(ClientPongPacket(reseed.id()))
            fixture.receive(ClientNameItemPacket("b"))

            menu.edits shouldContainExactly listOf(almost + "a", almost + "ab")
        }

        "a submit in flight after a reopen and a click submits the kept text" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            val almost = "a".repeat(49)
            menu.prompt(almost)
            val anvil = fixture.sent<OpenWindowPacket>().single().windowId()
            fixture.receive(ClientPongPacket(fixture.sent<PingPacket>().single().id()))

            fixture.clear()
            fixture.receive(ClientNameItemPacket(almost + "a"))
            fixture.receive(fixture.click(anvil, 1))
            fixture.receive(ClientNameItemPacket(almost))
            fixture.receive(fixture.click(anvil, 2))

            menu.prompts shouldContainExactly listOf(almost + "a")
            fixture.player.openInventory shouldBe null
        }

        "taking the result resyncs the player's experience" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.level = 3
            val menu = Menu(player)
            menu.open()
            menu.prompt("hi")
            val anvil = fixture.sent<OpenWindowPacket>().single().windowId()

            fixture.clear()
            fixture.receive(fixture.click(anvil, 2))

            fixture.sent<SetExperiencePacket>().single().level() shouldBe 3
        }

        "closing the anvil cancels the prompt on the next tick, and closing the window closes the anvil" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            val session = menu.open()

            menu.prompt("")
            player.closeInventory()
            menu.prompts shouldBe emptyList()
            fixture.tick()
            menu.prompts shouldContainExactly listOf(null)

            menu.prompt("")
            session.close()
            player.openInventory shouldBe null
            fixture.tick()
            menu.prompts shouldContainExactly listOf(null)
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
    }) {
    private companion object {
        const val HIDDEN_AGE = -1_000_000_001L
    }
}

private class Menu(
    player: Player,
) : FullscreenView(FullscreenDefinition(manifest(), "menu"), MinestomFullscreenHost(player)) {
    val clicks = mutableListOf<FullscreenClick>()
    val rightClicks = mutableListOf<FullscreenClick>()
    val scrolls = mutableListOf<Int>()
    val edits = mutableListOf<String>()
    val prompts = mutableListOf<String?>()
    var closed = false

    override fun FullscreenScope.bind() {
        button("press") { clicks += it }
        rightClick { rightClicks += it }
        scroll { scrolls += it }
    }

    fun prompt(initial: String) = promptText(initial = initial, onEdit = { edits += it }) { prompts += it }

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

    fun receive(packet: ClientPacket) = PlayerPacketEvent(player, packet).also(EventDispatcher::call)

    fun click(
        windowId: Int,
        slot: Int,
        changed: Map<Short, ItemStack.Hash> = emptyMap(),
    ) = ClientClickWindowPacket(
        windowId,
        0,
        slot.toShort(),
        0,
        ClientClickWindowPacket.ClickType.PICKUP,
        changed,
        ItemStack.Hash.AIR,
    )

    /** The paper in the first slot of window [windowId] as last sent. */
    fun seedItem(windowId: Int): ItemStack =
        sent<WindowItemsPacket>()
            .last { it.windowId() == windowId }
            .items()[0]

    /** The name of the paper in the first slot of window [windowId] as last sent. */
    fun seed(windowId: Int): String? {
        val name = seedItem(windowId).get(DataComponents.CUSTOM_NAME) ?: return null
        return PlainTextComponentSerializer.plainText().serialize(name)
    }

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

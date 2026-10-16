package com.chunkzero.window.minestom

import com.chunkzero.window.ExperimentalFullscreenApi
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
import io.kotest.matchers.ints.shouldBeLessThan
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotBe
import net.kyori.adventure.key.Key
import net.kyori.adventure.nbt.CompoundBinaryTag
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.coordinate.Vec
import net.minestom.server.dialog.Dialog
import net.minestom.server.dialog.DialogAction
import net.minestom.server.dialog.DialogInput
import net.minestom.server.entity.Entity
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.EquipmentSlot
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.PlayerHand
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.event.EventDispatcher
import net.minestom.server.event.EventListener
import net.minestom.server.event.inventory.InventoryOpenEvent
import net.minestom.server.event.player.PlayerCustomClickEvent
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.player.PlayerPacketOutEvent
import net.minestom.server.event.player.PlayerSpawnEvent
import net.minestom.server.inventory.Inventory
import net.minestom.server.inventory.InventoryType
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.ConnectionState
import net.minestom.server.network.packet.client.ClientPacket
import net.minestom.server.network.packet.client.common.ClientPongPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket
import net.minestom.server.network.packet.client.play.ClientClickWindowPacket.ClickType
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientHeldItemChangePacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientInteractEntityPacket
import net.minestom.server.network.packet.client.play.ClientNameItemPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.SendablePacket
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.common.ClearDialogPacket
import net.minestom.server.network.packet.server.common.PingPacket
import net.minestom.server.network.packet.server.common.ShowDialogPacket
import net.minestom.server.network.packet.server.play.CameraPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityEquipmentPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.HeldItemChangePacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.RespawnPacket
import net.minestom.server.network.packet.server.play.SetExperiencePacket
import net.minestom.server.network.packet.server.play.SetPassengersPacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.network.packet.server.play.WindowPropertyPacket
import net.minestom.server.network.player.GameProfile
import net.minestom.server.network.player.PlayerConnection
import net.minestom.server.registry.RegistryKey
import net.minestom.server.world.DimensionType
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

        "a prompt shows a dialog that keeps the HUD hidden, and Confirm delivers the field's text" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            fixture.tick()
            fixture.clear()

            menu.dialog("hi")
            val dialog = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction
            (dialog.metadata().inputs().single() as DialogInput.Text).initial() shouldBe "hi"
            dialog.metadata().canCloseWithEscape() shouldBe false
            fixture.sent<SetTimePacket>() shouldBe emptyList()

            fixture.click(dialog, 0, "text" to "hello")
            menu.prompts shouldContainExactly listOf("hello")
            fixture.click(dialog, 0, "text" to "again")
            menu.prompts shouldContainExactly listOf("hello")
        }

        "Cancel delivers null" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.dialog("hi")
            val dialog = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction

            fixture.click(dialog, 1)

            menu.prompts shouldContainExactly listOf(null)
        }

        "a response to another prompt's dialog or a foreign action is ignored" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.dialog("first")
            val first = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction
            menu.dialog("second")
            menu.prompts shouldContainExactly listOf(null)
            val second = fixture.sent<ShowDialogPacket>().last().dialog() as Dialog.MultiAction

            fixture.click(first, 0, "text" to "stale")
            fixture.respond(Key.key("other", "confirm"), "text" to "foreign")
            menu.prompts shouldContainExactly listOf(null)
            fixture.click(second, 0, "text" to "now")

            menu.prompts shouldContainExactly listOf(null, "now")
        }

        "a container opening over the dialog cancels the prompt on the next tick and re-enables the canvas" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.dialog("hi")
            val dialog = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction

            fixture.clickCanvas()
            menu.clicks shouldBe emptyList()
            fixture.send(OpenWindowPacket(1, InventoryType.CHEST_1_ROW.ordinal, Component.empty()))
            menu.prompts shouldBe emptyList()
            fixture.tick()
            fixture.click(dialog, 0, "text" to "late")
            fixture.clickCanvas()

            menu.prompts shouldContainExactly listOf(null)
            menu.clicks shouldContainExactly listOf(FullscreenClick(5, 5))
        }

        "a container opened before a prompt's dialog is written leaves that prompt pending" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            fixture.tick()
            fixture.clear()

            fixture.player.openInventory(Inventory(InventoryType.CHEST_1_ROW, Component.empty()))
            menu.dialog("hi")
            val dialog = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction
            fixture.tick()
            fixture.click(dialog, 0, "text" to "kept")

            menu.prompts shouldContainExactly listOf("kept")
        }

        "disconnecting with a prompt open reports null and closes once" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.dialog("hi")

            EventDispatcher.call(PlayerDisconnectEvent(fixture.player))
            EventDispatcher.call(PlayerDisconnectEvent(fixture.player))

            menu.prompts shouldContainExactly listOf(null)
            menu.closes shouldBe 1
        }

        "closing with a prompt open restores first, then reports null and onClose, even if the callback fails" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            val session = menu.open()
            menu.dialog("hi")
            val dialog = fixture.sent<ShowDialogPacket>().single().dialog() as Dialog.MultiAction
            fixture.clear()

            session.close()
            fixture.sent<ClearDialogPacket>().size shouldBe 1
            fixture.click(dialog, 0, "text" to "late")
            menu.prompts shouldContainExactly listOf(null)

            val sneaking = FullscreenFixture()
            val other = Menu(sneaking.player)
            other.open()
            var mode: GameMode? = null
            other.dialog("hi") {
                mode = sneaking.player.gameMode
                other.dialog("again")
            }
            sneaking.receive(ClientInputPacket(false, false, false, false, false, true, false))
            mode shouldBe GameMode.SURVIVAL
            other.prompts shouldContainExactly listOf(null, null)
            other.closed shouldBe true
            sneaking.sent<ShowDialogPacket>().size shouldBe 1

            val failing = FullscreenFixture()
            val broken = Menu(failing.player)
            val failed = broken.open()
            broken.dialog("hi") { error("boom") }
            shouldThrow<IllegalStateException> { failed.close() }
            failing.player.gameMode shouldBe GameMode.SURVIVAL
            broken.closed shouldBe true
        }

        "the keyboard opens an untitled anvil in the prompt band over blank slots, and Escape finishes with the text" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.inventory.setItemStack(2, ItemStack.of(Material.DIAMOND))
            val menu = Menu(player)
            menu.open()
            fixture.tick()
            fixture.clear()

            menu.type("hi")
            val open = fixture.sent<OpenWindowPacket>().single()
            open.windowType() shouldBe InventoryType.ANVIL.windowType
            open.title() shouldBe Component.empty()
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe PROMPT_AGE
            val items = fixture.sent<WindowItemsPacket>().last { it.windowId() == open.windowId() }.items()
            items[0].get(DataComponents.CUSTOM_NAME) shouldBe Component.text("hi")
            items.drop(3).map { it.get(DataComponents.ITEM_MODEL) }.distinct() shouldBe
                listOf("window:fullscreen/blank")
            fixture.sent<WindowPropertyPacket>().single().value() shouldBe 0.toShort()

            // The field's echo of its seed arrives before the pong and is not an edit.
            fixture.receive(ClientNameItemPacket("hi")).isCancelled shouldBe true
            fixture.pong()
            fixture.receive(ClientNameItemPacket("hey"))
            fixture.receive(ClientNameItemPacket("hey"))
            menu.edits shouldContainExactly listOf("hey")
            fixture.clickCanvas()
            menu.clicks shouldBe emptyList()

            fixture.clear()
            fixture.closeWindow(open.windowId())
            menu.prompts shouldContainExactly listOf("hey")
            player.openInventory shouldBe null
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
            fixture.sent<SetExperiencePacket>().size shouldBe 1
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
            fixture.clickCanvas()
            menu.clicks shouldContainExactly listOf(FullscreenClick(5, 5))
        }

        "a click outside the anvil finishes with the text and closes the anvil" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.type("hi")
            val window = fixture.window()
            fixture.clear()

            fixture.clickWindow(window, -999).isCancelled shouldBe true

            menu.prompts shouldContainExactly listOf("hi")
            fixture.player.openInventory shouldBe null
            fixture.sent<CloseWindowPacket>().single().windowId() shouldBe window
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
        }

        "a click carrying a stale window id is cancelled and leaves the prompt open" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.type("hi")
            val window = fixture.window()

            fixture.clickWindow(window + 1, 3).isCancelled shouldBe true

            menu.prompts shouldBe emptyList()
            fixture.player.openInventory shouldNotBe null
        }

        "a prompt replacing one of the other kind takes it off the client first; a keyboard keeps its anvil" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            menu.open()
            fixture.tick()
            fixture.clear()

            menu.type("a")
            val window = fixture.window()
            menu.type("b")
            menu.prompts shouldContainExactly listOf(null)
            fixture.sent<OpenWindowPacket>().size shouldBe 1
            fixture
                .sent<WindowItemsPacket>()
                .last()
                .items()[0]
                .get(DataComponents.CUSTOM_NAME) shouldBe
                Component.text("b")

            fixture.clear()
            menu.dialog("c")
            menu.prompts shouldContainExactly listOf(null, null)
            player.openInventory shouldBe null
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe HIDDEN_AGE
            fixture.sent<SetExperiencePacket>().size shouldBe 1
            val closed = fixture.delivered.indexOfFirst { it is CloseWindowPacket && it.windowId() == window }
            closed shouldBeLessThan fixture.delivered.indexOfFirst { it is ShowDialogPacket }

            fixture.clear()
            menu.type("d")
            menu.prompts shouldContainExactly listOf(null, null, null)
            fixture.sent<SetTimePacket>().single().gameTime() shouldBe PROMPT_AGE
            val cleared = fixture.delivered.indexOfFirst { it is ClearDialogPacket }
            cleared shouldBeLessThan fixture.delivered.indexOfFirst { it is OpenWindowPacket }
        }

        "ending the window mid-edit restores the band, slots, and XP, then reports null once before onClose" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            player.inventory.setItemStack(2, ItemStack.of(Material.DIAMOND))
            val menu = Menu(player)
            menu.open()
            fixture.tick()
            var mode: GameMode? = null
            menu.type("hi") { mode = player.gameMode }
            fixture.clear()

            fixture.receive(ClientInputPacket(false, false, false, false, false, true, false))

            mode shouldBe GameMode.SURVIVAL
            menu.prompts shouldContainExactly listOf(null)
            menu.closes shouldBe 1
            player.openInventory shouldBe null
            fixture.sent<CloseWindowPacket>().size shouldBe 1
            fixture.sent<SetTimePacket>().last().gameTime() shouldBe player.instance.worldAge
            fixture.sent<SetExperiencePacket>().size shouldBe 1
            fixture.hotbar()[2] shouldBe Material.DIAMOND
            fixture.tick()
            menu.prompts shouldContainExactly listOf(null)
        }

        "a server container replacing the anvil resolves null on the next tick and re-enables the canvas" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            menu.open()
            menu.type("hi")
            fixture.clear()

            player.openInventory(Inventory(InventoryType.CHEST_1_ROW, Component.empty()))
            menu.prompts shouldBe emptyList()
            fixture.sent<SetTimePacket>().last() shouldBe player.instance.createTimePacket()
            fixture.tick()
            fixture.clickCanvas()

            menu.prompts shouldContainExactly listOf(null)
            menu.clicks shouldContainExactly listOf(FullscreenClick(5, 5))
        }

        "the keyboard holds 50 characters: a longer initial text is cut, and a longer rename is truncated" {
            val fixture = FullscreenFixture()
            val menu = Menu(fixture.player)
            menu.open()
            menu.type("x".repeat(55))
            val window = fixture.window()
            fixture
                .sent<WindowItemsPacket>()
                .last()
                .items()[0]
                .get(DataComponents.CUSTOM_NAME) shouldBe
                Component.text("x".repeat(50))
            fixture.pong()

            fixture.receive(ClientNameItemPacket("x".repeat(50) + "y"))
            menu.edits shouldBe emptyList()
            fixture.receive(ClientNameItemPacket("z".repeat(60)))
            fixture.closeWindow(window)

            menu.edits shouldContainExactly listOf("z".repeat(50))
            menu.prompts shouldContainExactly listOf("z".repeat(50))
        }

        "a refused anvil open resolves the prompt with null and leaves the band hidden" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            menu.open()
            player.eventNode().addListener(InventoryOpenEvent::class.java) { it.isCancelled = true }
            fixture.clear()

            menu.type("hi")

            menu.prompts shouldContainExactly listOf(null)
            player.openInventory shouldBe null
            fixture.sent<SetTimePacket>().last().gameTime() shouldBe HIDDEN_AGE
        }

        "a keyboard replacing an open container takes over its band, and finishing restores the hidden band and slots" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val diamond = ItemStack.of(Material.DIAMOND)
            player.inventory.setItemStack(2, diamond)
            val menu = Menu(player)
            menu.open()
            player.openInventory(Inventory(InventoryType.CHEST_1_ROW, Component.empty()))
            fixture.clear()
            fixture.hotbar()[2] shouldBe Material.DIAMOND

            menu.type("hi")
            fixture.sent<SetTimePacket>().last().gameTime() shouldBe PROMPT_AGE
            val window = fixture.window()

            fixture.clear()
            fixture.closeWindow(window)
            menu.prompts shouldContainExactly listOf("hi")
            fixture.sent<SetTimePacket>().last().gameTime() shouldBe HIDDEN_AGE
            fixture.hotbar() shouldBe List(9) { Material.PAPER }

            fixture.clear()
            fixture.send(SetTimePacket(100, emptyMap()))
            fixture.sent<SetTimePacket>().map { it.gameTime() } shouldContainExactly listOf(100L, HIDDEN_AGE)
            fixture.send(SetPlayerInventorySlotPacket(2, diamond))
            fixture.hotbar() shouldBe List(9) { Material.PAPER }
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
        "a player riding a vehicle is put back on it when horse mode closes" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val cart = Entity(EntityType.MINECART)
            cart.setInstance(player.instance, player.position).join()
            cart.addPassenger(player)
            val session = Menu(player, FullscreenMode.HORSE).open()
            fixture.tick()

            fixture.clear()
            session.close()

            fixture.sent<SetPassengersPacket>().last() shouldBe
                SetPassengersPacket(cart.entityId, listOf(player.entityId))
        }

        for ((name, dimension) in mapOf("within a dimension" to "overworld", "across dimensions" to "the_nether")) {
            "changing instance $name ends the window without teleporting the player back" {
                val fixture = FullscreenFixture()
                val player = fixture.player
                val menu = Menu(player, FullscreenMode.HORSE)
                menu.open()
                fixture.tick()

                val elsewhere = Pos(100.0, 80.0, 100.0)
                val other =
                    MinecraftServer.getInstanceManager().createInstanceContainer(
                        RegistryKey.unsafeOf<DimensionType>("minecraft:$dimension"),
                    )
                fixture.clear()
                player.setInstance(other, elsewhere).join()

                menu.closes shouldBe 1
                player.gameMode shouldBe GameMode.SURVIVAL
                player.position shouldBe elsewhere
                fixture.sent<RespawnPacket>().size shouldBe if (dimension == "overworld") 0 else 1
                fixture.sent<CameraPacket>().single().cameraId() shouldBe player.entityId
                fixture.sent<DestroyEntitiesPacket>().flatMap { it.entityIds() }.size shouldBe 5

                // The close already happened, so leaving the server right after it does not report another.
                EventDispatcher.call(PlayerDisconnectEvent(player))
                menu.closes shouldBe 1
            }
        }

        "changing instance in the tick the window opened ends it" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            menu.open()
            val elsewhere = Pos(100.0, 80.0, 100.0)

            player.setInstance(MinecraftServer.getInstanceManager().createInstanceContainer(), elsewhere).join()
            fixture.tick()

            menu.closes shouldBe 1
            player.gameMode shouldBe GameMode.SURVIVAL
            player.position shouldBe elsewhere
        }

        "death ends the window where the player died, and the respawn that follows leaves it ended" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            menu.open()
            fixture.tick()
            val destination = Pos(100.0, 80.0, 100.0)
            player.respawnPoint = destination

            player.kill()
            menu.closes shouldBe 1
            player.gameMode shouldBe GameMode.SURVIVAL
            player.position shouldBe Pos(0.0, 42.0, 0.0)
            shouldThrow<IllegalStateException> { Menu(player).open() }

            player.respawn()
            fixture.tick()
            menu.closes shouldBe 1
            player.position shouldBe destination
            player.gameMode shouldBe GameMode.SURVIVAL
        }

        "a window opened by a spawn listener is not ended by the spawn that opened it" {
            val fixture = FullscreenFixture()
            val player = fixture.player
            val menu = Menu(player)
            // Minestom reaches a window's listeners from an earlier listener only if the player's node listens already.
            player.eventNode().addListener(PlayerSpawnEvent::class.java) { }
            val opener =
                EventListener.of(PlayerSpawnEvent::class.java) {
                    menu
                        .takeIf { _ ->
                            it.player == player
                        }?.open()
                }
            MinecraftServer.getGlobalEventHandler().addListener(opener)
            val elsewhere = Pos(100.0, 80.0, 100.0)

            player.setInstance(MinecraftServer.getInstanceManager().createInstanceContainer(), elsewhere).join()
            MinecraftServer.getGlobalEventHandler().removeListener(opener)

            menu.closes shouldBe 0
            player.gameMode shouldBe GameMode.SPECTATOR
            player.position shouldBe elsewhere.withView(0f, 0f)
        }
    }) {
    private companion object {
        const val HIDDEN_AGE = -1_000_000_001L
        const val PROMPT_AGE = -999_795_201L
    }
}

@OptIn(ExperimentalFullscreenApi::class)
private class Menu(
    player: Player,
    mode: FullscreenMode = FullscreenMode.UNLOCKED,
) : FullscreenView(FullscreenDefinition(manifest(mode), "menu"), MinestomFullscreenHost(player)) {
    val clicks = mutableListOf<FullscreenClick>()
    val rightClicks = mutableListOf<FullscreenClick>()
    val scrolls = mutableListOf<Int>()
    val prompts = mutableListOf<String?>()
    val edits = mutableListOf<String>()
    var closes = 0
    val closed get() = closes > 0

    override fun FullscreenScope.bind() {
        button("press") { clicks += it }
        rightClick { rightClicks += it }
        scroll { scrolls += it }
    }

    fun dialog(
        initial: String,
        then: () -> Unit = {},
    ) = promptTextDialog(initial = initial) {
        prompts += it
        then()
    }

    fun type(
        initial: String = "",
        then: () -> Unit = {},
    ) = promptText(initial, onEdit = { edits += it }) {
        prompts += it
        then()
    }

    override fun onClose() {
        closes++
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

    /** Clicks the menu's button, at canvas pixel (5, 5), and runs the end of the player's tick. */
    fun clickCanvas() {
        receive(ClientAttackPacket(0))
        receive(ClientPlayerRotationPacket(-44f, -24.375f, 0.toByte()))
        player.scheduler().processTickEnd()
    }

    fun receive(packet: ClientPacket) = PlayerPacketEvent(player, packet).also(EventDispatcher::call)

    /** The window id of the latest window the client was told to open. */
    fun window(): Int = sent<OpenWindowPacket>().last().windowId()

    /** Answers the latest ping, which settles the keyboard's latest seed. */
    fun pong() {
        receive(ClientPongPacket(sent<PingPacket>().last().id()))
    }

    /** Closes window [id] from the client, as Escape does, then lets Minestom close it unless the event was cancelled. */
    fun closeWindow(id: Int) {
        if (!receive(ClientCloseWindowPacket(id)).isCancelled) player.closeInventory(true, id.toByte())
    }

    /** Clicks [slot] of window [id], -999 being outside it. */
    fun clickWindow(
        id: Int,
        slot: Int,
    ) = receive(ClientClickWindowPacket(id, 0, slot.toShort(), 0, ClickType.PICKUP, emptyMap(), ItemStack.Hash.AIR))

    /** Presses [dialog]'s button [index], answering with [fields]. */
    fun click(
        dialog: Dialog.MultiAction,
        index: Int,
        vararg fields: Pair<String, String>,
    ) = respond((dialog.actions()[index].action() as DialogAction.DynamicCustom).key(), *fields)

    fun respond(
        key: Key,
        vararg fields: Pair<String, String>,
    ) {
        val payload = CompoundBinaryTag.builder()
        for ((name, value) in fields) payload.putString(name, value)
        EventDispatcher.call(PlayerCustomClickEvent(player, key, payload.build()))
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
            promptWorldAge = -999_795_201,
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

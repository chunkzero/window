package com.chunkzero.window.minestom.internal

import com.chunkzero.window.FullscreenDefinition
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.coordinate.Vec
import net.minestom.server.entity.Entity
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.PlayerHand
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.entity.metadata.display.AbstractDisplayMeta
import net.minestom.server.entity.metadata.display.TextDisplayMeta
import net.minestom.server.entity.metadata.other.InteractionMeta
import net.minestom.server.event.EventFilter
import net.minestom.server.event.EventNode
import net.minestom.server.event.player.PlayerCustomClickEvent
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerMoveEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.player.PlayerPacketOutEvent
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.packet.client.play.ClientAnimationPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientHeldItemChangePacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientInteractEntityPacket
import net.minestom.server.network.packet.client.play.ClientPlayerActionPacket
import net.minestom.server.network.packet.client.play.ClientPlayerPositionAndRotationPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.ServerPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.HeldItemChangePacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.PlayerRotationPacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetSlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
import net.minestom.server.network.packet.server.play.UpdateHealthPacket
import net.minestom.server.network.packet.server.play.WindowItemsPacket
import net.minestom.server.registry.RegistryKey
import net.minestom.server.tag.Tag
import net.minestom.server.world.clock.WorldClock

/**
 * A full-screen window shown to [player] as a client-only text display at their eye, reporting their input through a
 * child of their event node.
 *
 * The server keeps the player in spectator mode, pinned at their position facing yaw and pitch 0. Their client alone is
 * told it is in survival, flying at zero speed, so the scroll wheel and number keys change the held slot, which is
 * reported as steps and never applied to the server's player. A client-only interaction entity encloses the eye, so
 * every left click attacks it without a miss cooldown, and every right click interacts with it once, with the main hand.
 *
 * The HUD is hidden: the hotbar holds blank items, which also hide the hand, and the client's world age is held
 * negative, which the pack's GUI shader reads as "clip the bottom-centre HUD strip". So the hearts fit that strip, the
 * client is told its max health is at most the canvas's `hudMaxHealth` and its absorption is 0. While the server has a
 * container open, the client gets the real world age back and its real hotbar so the strip and slots show. The text
 * prompt's dialog is a client screen rather than a server container, so it keeps the HUD hidden and the hotbar blank.
 *
 * While the window is open, a server update to the game mode, abilities, hotbar, time, max health, or absorption passes
 * through unchanged and is followed by a packet that puts the hidden state back. Replacing the update instead would
 * queue the replacement behind every packet already waiting, such as an older hotbar snapshot overwriting a newer
 * slot. The follow-ups run on the connection's writer thread, so [lock] guards the lifecycle and container state:
 * once the window stops, none can enqueue a packet after the restoring ones.
 *
 * The client advances every world clock by the world age delta of each time packet before applying the packet's clock
 * snapshots, so every packet that changes the world age carries the real clock values, or the hidden age would move
 * the clocks with it.
 *
 * A prompt's callback runs on the player's thread after the state it reports has changed, never under [lock]: a server
 * container opening over the dialog cancels it on the next tick, and a window ending cancels it once the player is
 * restored, just before [FullscreenListener.onClose].
 */
internal class MinestomFullscreen private constructor(
    private val player: Player,
    definition: FullscreenDefinition,
    private val listener: FullscreenListener,
) : OpenFullscreen {
    private val canvas = definition.canvas
    private val blank =
        ItemStack
            .builder(Material.PAPER)
            .set(DataComponents.ITEM_MODEL, definition.blankItemModel.asString())
            .set(DataComponents.CUSTOM_NAME, Component.empty())
            .build()
    private val gameMode = player.gameMode
    private val allowFlying = player.isAllowFlying
    private val flying = player.isFlying
    private val invulnerable = player.isInvulnerable
    private val instantBreak = player.isInstantBreak
    private val flyingSpeed = player.flyingSpeed
    private val position = player.position
    private val heldSlot = player.heldSlot
    private val anchor = position.withView(0f, 0f)
    private val display = Entity(EntityType.TEXT_DISPLAY)
    private val clickTarget = Entity(EntityType.INTERACTION)
    private val node = EventNode.type("window-fullscreen", EventFilter.PLAYER)
    private val survival = ChangeGameStatePacket(ChangeGameStatePacket.Reason.CHANGE_GAMEMODE, 0f)
    private val abilities = PlayerAbilitiesPacket(ABILITY_FLAGS, 0f, WALKING_SPEED)
    private val lock = Any()
    private var open = false

    /** The held slot the client has selected, which held-slot changes are measured from. Requires [lock]. */
    private var clientSlot = heldSlot.toInt()
    private val textPrompt = MinestomTextPrompt(player)

    /** The window id of the server container the client has open, or [NO_CONTAINER]. */
    private var containerWindowId = NO_CONTAINER
    private val containerOpen get() = containerWindowId != NO_CONTAINER

    /** Changes whenever a container opens or closes, so a callback deferred past a tick can tell it is outdated. */
    private var containerGeneration = 0

    override val latencyMillis: Int
        get() = player.latency

    override fun show(content: Component) {
        display.editEntityMeta(TextDisplayMeta::class.java) { it.text = content }
        player.sendPacket(display.metadataPacket)
    }

    override fun turn(
        yaw: Float,
        pitch: Float,
    ) {
        player.sendPacket(PlayerRotationPacket(yaw, true, pitch, true))
    }

    override val promptOpen: Boolean
        get() = textPrompt.current != null

    override fun promptText(
        title: Component,
        initial: String,
        onDone: (String?) -> Unit,
    ) {
        if (isOpen()) textPrompt.open(title, initial, onDone) else onDone(null)
    }

    override fun close() {
        if (!stop()) return
        try {
            restore()
        } finally {
            textPrompt.cancel()
        }
    }

    private fun start(content: Component) {
        check(!player.hasTag(ACTIVE)) { "${player.username} already has a full-screen window open" }
        checkNotNull(player.instance) { "${player.username} is in no instance" }
        check(player.setGameMode(GameMode.SPECTATOR)) {
            "Opening a full-screen window for ${player.username} was cancelled by a game mode listener"
        }
        synchronized(lock) {
            open = true
            containerWindowId = player.openInventory?.windowId?.toInt() ?: NO_CONTAINER
        }
        player.setTag(ACTIVE, true)
        player.flyingSpeed = 0f
        player.teleport(anchor)
        // The game mode change also updated the client's own tab-list entry, so survival is faked after it.
        player.scheduler().scheduleNextTick { whileOpen { fakeSurvival() } }
        spawn(content)
        if (!containerOpen) blankHotbar()
        hideHud()
        listen()
    }

    /**
     * Tells the client alone it is in survival. The game event sets its interaction mode, and its own tab-list entry
     * decides `isSpectator`, which would turn the scroll wheel into a flying speed change. Survival drops the client's
     * flight, so flight is granted again at zero speed.
     */
    private fun fakeSurvival() {
        player.sendPacket(survival)
        val entry =
            PlayerInfoUpdatePacket.Entry(
                player.uuid,
                player.username,
                emptyList(),
                false,
                0,
                GameMode.SURVIVAL,
                null,
                null,
                0,
                false,
            )
        player.sendPacket(PlayerInfoUpdatePacket(PlayerInfoUpdatePacket.Action.UPDATE_GAME_MODE, entry))
        player.sendPacket(abilities)
    }

    private fun spawn(content: Component) {
        display.editEntityMeta(TextDisplayMeta::class.java) { meta ->
            meta.text = content
            meta.lineWidth = Int.MAX_VALUE
            meta.backgroundColor = 0
            meta.textOpacity = canvas.markerAlpha.toByte()
            meta.isShadow = false
            meta.alignment = TextDisplayMeta.Alignment.LEFT
            meta.billboardRenderConstraints = AbstractDisplayMeta.BillboardConstraints.FIXED
            meta.setBrightness(15, 15)
            meta.scale = Vec(canvas.displayScale)
            // A zero-sized culling box turns off frustum culling for display entities.
            meta.width = 0f
            meta.height = 0f
        }
        clickTarget.editEntityMeta(InteractionMeta::class.java) { meta ->
            meta.width = CLICK_BOX
            meta.height = CLICK_BOX
        }
        val eye = anchor.add(0.0, player.eyeHeight, 0.0)
        spawn(display, eye)
        spawn(clickTarget, eye.sub(0.0, CLICK_BOX / 2.0, 0.0))
    }

    private fun spawn(
        entity: Entity,
        at: Pos,
    ) {
        player.sendPacket(SpawnEntityPacket(entity.entityId, entity.uuid, entity.entityType, at, 0f, 0, Vec.ZERO))
        player.sendPacket(entity.metadataPacket)
    }

    private fun blankHotbar() {
        blank(0 until HOTBAR_SIZE)
    }

    private fun blank(slots: Iterable<Int>) {
        for (slot in slots) player.sendPacket(SetPlayerInventorySlotPacket(slot, blank))
    }

    private fun hideHud() {
        if (!containerOpen) hideTime(realTime().clocks())
        capMaxHealth()
        player.sendPacket(EntityMetaDataPacket(player.entityId, mapOf(ABSORPTION to ZERO)))
    }

    private fun hideTime(clocks: Map<RegistryKey<WorldClock>, SetTimePacket.ClockState>) {
        player.sendPacket(SetTimePacket(canvas.hudHiddenWorldAge, clocks))
    }

    private fun realTime() = player.instance.createTimePacket()

    private fun capMaxHealth() {
        cappedMaxHealth()?.let { player.sendPacket(EntityAttributesPacket(player.entityId, listOf(it))) }
    }

    /** The modifier-free max health the client is told while the HUD is hidden, or `null` when the real one fits. */
    private fun cappedMaxHealth(): EntityAttributesPacket.Property? {
        val cap = canvas.hudMaxHealth.toDouble()
        if (player.getAttribute(Attribute.MAX_HEALTH).value <= cap) return null
        return EntityAttributesPacket.Property(Attribute.MAX_HEALTH, cap, emptyList())
    }

    private fun listen() {
        node.addListener(PlayerPacketEvent::class.java, ::onPacket)
        node.addListener(PlayerPacketOutEvent::class.java, ::onPacketOut)
        node.addListener(PlayerMoveEvent::class.java) { event ->
            val next = event.newPosition
            if (!next.samePoint(anchor)) event.newPosition = anchor.withView(next.yaw(), next.pitch())
        }
        node.addListener(PlayerCustomClickEvent::class.java, textPrompt::onClick)
        node.addListener(PlayerDisconnectEvent::class.java) { end(restore = false) }
        player.eventNode().addChild(node)
    }

    private fun onPacket(event: PlayerPacketEvent) {
        when (val packet = event.packet) {
            is ClientPlayerRotationPacket -> {
                listener.onLook(packet.yaw(), packet.pitch())
            }

            is ClientPlayerPositionAndRotationPacket -> {
                listener.onLook(packet.position().yaw(), packet.position().pitch())
            }

            is ClientAttackPacket -> {
                event.isCancelled = true
                // The client sends a click before the same tick's rotation.
                player.scheduler().scheduleEndOfTick { if (isOpen()) listener.onClick() }
            }

            is ClientInteractEntityPacket -> {
                event.isCancelled = true
                // The interaction entity consumes a main-hand use, so the client never tries the off hand.
                if (packet.hand() == PlayerHand.MAIN) {
                    player.scheduler().scheduleEndOfTick { if (isOpen()) listener.onRightClick() }
                }
            }

            is ClientHeldItemChangePacket -> {
                event.isCancelled = true
                scroll(packet.slot().toInt())
            }

            // Dropping or swapping hands would resync the real hotbar over the blank items.
            is ClientAnimationPacket, is ClientPlayerActionPacket -> {
                event.isCancelled = true
            }

            is ClientCloseWindowPacket -> {
                // Minestom ignores a close for any window but the open one, so a stale close must not hide the HUD.
                whileOpen { if (packet.windowId() == containerWindowId) containerClosed() }
            }

            is ClientInputPacket -> {
                if (packet.shift()) end(restore = true)
            }

            else -> {
                Unit
            }
        }
    }

    /** Reports the client selecting [slot] as steps the shortest way around the hotbar from the slot it had. */
    private fun scroll(slot: Int) {
        if (slot !in 0 until HOTBAR_SIZE) return
        val steps =
            synchronized(lock) {
                val from = clientSlot
                clientSlot = slot
                Math.floorMod(slot - from + HOTBAR_SIZE / 2, HOTBAR_SIZE) - HOTBAR_SIZE / 2
            }
        if (steps != 0) listener.onScroll(steps)
    }

    /** Keeps the HUD hidden and the client in fake survival after each of the server's own updates. */
    private fun onPacketOut(event: PlayerPacketOutEvent) {
        whileOpen {
            when (val packet = event.packet) {
                is SetTimePacket -> {
                    if (!containerOpen && packet.gameTime() >= 0) hideTime(packet.clocks())
                }

                is EntityAttributesPacket -> {
                    val capped = cappedMaxHealth() ?: return
                    val uncapped = packet.properties().any { it.attribute() == Attribute.MAX_HEALTH && it != capped }
                    if (packet.entityId() == player.entityId && uncapped) capMaxHealth()
                }

                is EntityMetaDataPacket -> {
                    val absorption = packet.entries()[ABSORPTION]?.value()
                    if (packet.entityId() == player.entityId && absorption != null && absorption != 0f) {
                        player.sendPacket(EntityMetaDataPacket(player.entityId, mapOf(ABSORPTION to ZERO)))
                    }
                }

                is ChangeGameStatePacket -> {
                    if (packet.reason() == survival.reason() && packet.value() != survival.value()) fakeSurvival()
                }

                is PlayerInfoUpdatePacket -> {
                    val changesMode = PlayerInfoUpdatePacket.Action.UPDATE_GAME_MODE in packet.actions()
                    if (changesMode &&
                        packet.entries().any { it.uuid() == player.uuid && it.gameMode() != GameMode.SURVIVAL }
                    ) {
                        fakeSurvival()
                    }
                }

                is PlayerAbilitiesPacket -> {
                    if (packet != abilities) player.sendPacket(abilities)
                }

                is HeldItemChangePacket -> {
                    clientSlot = packet.slot().toInt()
                }

                is SetPlayerInventorySlotPacket, is SetSlotPacket, is WindowItemsPacket -> {
                    if (!containerOpen) blank(realHotbarSlots(packet))
                }

                is OpenWindowPacket -> {
                    containerWindowId = packet.windowId()
                    containerGeneration++
                    // The container replaces the client's dialog without any answer.
                    textPrompt.current?.let { covered ->
                        player.scheduler().scheduleNextTick { textPrompt.cancel(covered) }
                    }
                    player.sendPacket(realTime())
                    // Opening a container does not resync the player inventory, whose hotbar still holds the blanks.
                    player.inventory.update(player)
                }

                is CloseWindowPacket -> {
                    // The close packet is sent after this event, so the hidden time must follow it.
                    // Minestom reuses an inventory's window id, so a reopen is told apart by the generation.
                    val window = packet.windowId()
                    val generation = ++containerGeneration
                    player.scheduler().scheduleNextTick {
                        whileOpen {
                            if (generation == containerGeneration && window == containerWindowId) containerClosed()
                        }
                    }
                }

                else -> {
                    Unit
                }
            }
        }
    }

    /** The hotbar slots [packet] sets to something other than the blank. */
    private fun realHotbarSlots(packet: ServerPacket): List<Int> =
        when {
            packet is SetPlayerInventorySlotPacket && packet.slot() in 0 until HOTBAR_SIZE &&
                packet.itemStack() != blank -> {
                listOf(packet.slot())
            }

            packet is SetSlotPacket && packet.windowId() == 0 && packet.slot().toInt() in WINDOW_HOTBAR &&
                packet.itemStack() != blank -> {
                listOf(packet.slot() - WINDOW_HOTBAR.first)
            }

            packet is WindowItemsPacket && packet.windowId() == 0 -> {
                (0 until HOTBAR_SIZE).filter { slot ->
                    packet.items().getOrNull(WINDOW_HOTBAR.first + slot)?.let { it != blank } == true
                }
            }

            else -> {
                emptyList()
            }
        }

    /** Hides the HUD again once the server's container is gone. Requires [lock]. */
    private fun containerClosed() {
        containerWindowId = NO_CONTAINER
        val generation = ++containerGeneration
        hideTime(realTime().clocks())
        // Closing a container resyncs the real hotbar after this tick's packets, unless another container opened since.
        player.scheduler().scheduleNextTick { whileOpen { if (generation == containerGeneration) blankHotbar() } }
    }

    private fun isOpen() = synchronized(lock) { open }

    /** Runs [action] under [lock] unless the window has stopped, so it cannot enqueue a packet after restoring. */
    private inline fun whileOpen(action: () -> Unit) {
        synchronized(lock) { if (open) action() }
    }

    /** Ends the window after the player left it, restoring them unless they disconnected. */
    private fun end(restore: Boolean) {
        if (!stop()) return
        try {
            if (restore) restore()
        } finally {
            try {
                textPrompt.cancel()
            } finally {
                listener.onClose()
            }
        }
    }

    /** Stops listening; false when already stopped. */
    private fun stop(): Boolean =
        synchronized(lock) {
            if (!open) return false
            open = false
            player.eventNode().removeChild(node)
            player.removeTag(ACTIVE)
            true
        }

    private fun restore() {
        textPrompt.clear()
        player.sendPacket(DestroyEntitiesPacket(listOf(display.entityId, clickTarget.entityId)))
        // A mode the server set while the window was open replaces the one it began with.
        val mode = player.gameMode.takeUnless { it == GameMode.SPECTATOR } ?: gameMode
        player.setGameMode(mode)
        if (mode == gameMode) {
            player.isAllowFlying = allowFlying
            player.isFlying = flying
            player.isInvulnerable = invulnerable
            player.isInstantBreak = instantBreak
        }
        player.flyingSpeed = flyingSpeed
        player.teleport(position)
        player.sendPacket(realTime())
        val maxHealth = player.getAttribute(Attribute.MAX_HEALTH)
        val property = EntityAttributesPacket.Property(Attribute.MAX_HEALTH, maxHealth.baseValue, maxHealth.modifiers())
        player.sendPacket(EntityAttributesPacket(player.entityId, listOf(property)))
        player.sendPacket(player.metadataPacket)
        player.sendPacket(UpdateHealthPacket(player.health, player.food, player.foodSaturation))
        player.setHeldItemSlot(heldSlot)
        player.inventory.update()
    }

    companion object {
        private val ACTIVE = Tag.Transient<Boolean>("window:fullscreen")
        private val ABSORPTION = MetadataDef.Player.ADDITIONAL_HEARTS.index()
        private val ZERO = Metadata.Float(0f)
        private const val HOTBAR_SIZE = 9
        private const val NO_CONTAINER = -1

        /** The hotbar's slots in the player inventory window, which follows crafting, armor, and the main grid. */
        private val WINDOW_HOTBAR = 36..44
        private val ABILITY_FLAGS =
            (
                PlayerAbilitiesPacket.FLAG_INVULNERABLE.toInt() or
                    PlayerAbilitiesPacket.FLAG_FLYING.toInt() or
                    PlayerAbilitiesPacket.FLAG_ALLOW_FLYING.toInt()
            ).toByte()
        private const val WALKING_SPEED = 0.1f

        /** The side of the interaction cube centred on the eye. */
        private const val CLICK_BOX = 4f

        /**
         * Shows [definition] drawn by [content] to [player], then listens.
         *
         * @throws IllegalStateException if the player already has a full-screen window open, or a game mode listener
         *   cancelled the switch to spectator mode.
         */
        fun open(
            player: Player,
            definition: FullscreenDefinition,
            content: Component,
            listener: FullscreenListener,
        ): MinestomFullscreen = MinestomFullscreen(player, definition, listener).also { it.start(content) }
    }
}

package com.chunkzero.window.minestom.internal

import com.chunkzero.window.FullscreenDefinition
import com.chunkzero.window.host.FullscreenListener
import com.chunkzero.window.host.OpenFullscreen
import net.kyori.adventure.text.Component
import net.minestom.server.MinecraftServer
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.coordinate.Vec
import net.minestom.server.entity.Entity
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.GameMode
import net.minestom.server.entity.Metadata
import net.minestom.server.entity.MetadataDef
import net.minestom.server.entity.Player
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.entity.metadata.display.AbstractDisplayMeta
import net.minestom.server.entity.metadata.display.TextDisplayMeta
import net.minestom.server.entity.metadata.other.InteractionMeta
import net.minestom.server.event.EventFilter
import net.minestom.server.event.EventNode
import net.minestom.server.event.player.PlayerDisconnectEvent
import net.minestom.server.event.player.PlayerMoveEvent
import net.minestom.server.event.player.PlayerPacketEvent
import net.minestom.server.event.player.PlayerPacketOutEvent
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.packet.client.play.ClientAnimationPacket
import net.minestom.server.network.packet.client.play.ClientAttackPacket
import net.minestom.server.network.packet.client.play.ClientCloseWindowPacket
import net.minestom.server.network.packet.client.play.ClientInputPacket
import net.minestom.server.network.packet.client.play.ClientInteractEntityPacket
import net.minestom.server.network.packet.client.play.ClientPlayerActionPacket
import net.minestom.server.network.packet.client.play.ClientPlayerPositionAndRotationPacket
import net.minestom.server.network.packet.client.play.ClientPlayerRotationPacket
import net.minestom.server.network.packet.server.play.ChangeGameStatePacket
import net.minestom.server.network.packet.server.play.CloseWindowPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityMetaDataPacket
import net.minestom.server.network.packet.server.play.OpenWindowPacket
import net.minestom.server.network.packet.server.play.PlayerAbilitiesPacket
import net.minestom.server.network.packet.server.play.PlayerInfoUpdatePacket
import net.minestom.server.network.packet.server.play.PlayerRotationPacket
import net.minestom.server.network.packet.server.play.SetPlayerInventorySlotPacket
import net.minestom.server.network.packet.server.play.SetTimePacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket
import net.minestom.server.network.packet.server.play.UpdateHealthPacket
import net.minestom.server.tag.Tag

/**
 * A full-screen window shown to [player] as a client-only text display at their eye, reporting their input through a
 * child of their event node.
 *
 * The server keeps the player in spectator mode, pinned at their position facing yaw and pitch 0. Their client alone is
 * told it is in survival, flying at zero speed, so the scroll wheel and number keys change the held slot. A client-only
 * interaction entity encloses the eye, so every left click attacks it without a miss cooldown.
 *
 * The HUD is hidden: the hotbar holds blank items, which also hide the hand, and the client's world age is held
 * negative, which the pack's GUI shader reads as "clip the bottom-centre HUD strip". So the hearts fit that strip, the
 * client is told its max health is at most the canvas's `hudMaxHealth` and its absorption is 0. While the server has a
 * container open, the client gets the real world age back so the strip does not clip the container.
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
    private val flyingSpeed = player.flyingSpeed
    private val position = player.position
    private val heldSlot = player.heldSlot
    private val anchor = position.withView(0f, 0f)
    private val display = Entity(EntityType.TEXT_DISPLAY)
    private val clickTarget = Entity(EntityType.INTERACTION)
    private val node = EventNode.type("window-fullscreen", EventFilter.PLAYER)
    private var open = false
    private var containerOpen = false

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

    override fun close() {
        if (stop()) restore()
    }

    private fun start(content: Component) {
        check(!player.hasTag(ACTIVE)) { "${player.username} already has a full-screen window open" }
        checkNotNull(player.instance) { "${player.username} is in no instance" }
        check(player.setGameMode(GameMode.SPECTATOR)) {
            "Opening a full-screen window for ${player.username} was cancelled by a game mode listener"
        }
        open = true
        player.setTag(ACTIVE, true)
        player.flyingSpeed = 0f
        player.teleport(anchor)
        // The game mode change also updated the client's own tab-list entry, so survival is faked after it.
        player.scheduler().scheduleNextTick { if (open) fakeSurvival() }
        spawn(content)
        blankHotbar()
        hideHud()
        listen()
    }

    /**
     * Tells the client alone it is in survival. The game event sets its interaction mode, and its own tab-list entry
     * decides `isSpectator`, which would turn the scroll wheel into a flying speed change. Survival drops the client's
     * flight, so flight is granted again at zero speed.
     */
    private fun fakeSurvival() {
        player.sendPacket(
            ChangeGameStatePacket(ChangeGameStatePacket.Reason.CHANGE_GAMEMODE, GameMode.SURVIVAL.ordinal.toFloat()),
        )
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
        val flags =
            PlayerAbilitiesPacket.FLAG_INVULNERABLE.toInt() or
                PlayerAbilitiesPacket.FLAG_FLYING.toInt() or
                PlayerAbilitiesPacket.FLAG_ALLOW_FLYING.toInt()
        player.sendPacket(PlayerAbilitiesPacket(flags.toByte(), 0f, WALKING_SPEED))
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
        for (slot in 0 until HOTBAR_SIZE) player.sendPacket(SetPlayerInventorySlotPacket(slot, blank))
    }

    private fun hideHud() {
        player.sendPacket(SetTimePacket(canvas.hudHiddenWorldAge, emptyMap()))
        cappedMaxHealth()?.let { player.sendPacket(EntityAttributesPacket(player.entityId, listOf(it))) }
        player.sendPacket(EntityMetaDataPacket(player.entityId, mapOf(ABSORPTION to Metadata.Float(0f))))
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
                MinecraftServer.getSchedulerManager().scheduleEndOfTick { if (open) listener.onClick() }
            }

            // Dropping or swapping hands would resync the real hotbar over the blank items.
            is ClientAnimationPacket, is ClientInteractEntityPacket, is ClientPlayerActionPacket -> {
                event.isCancelled = true
            }

            is ClientCloseWindowPacket -> {
                containerClosed()
            }

            is ClientInputPacket -> {
                if (packet.shift()) end(restore = true)
            }

            else -> {
                Unit
            }
        }
    }

    /** Keeps the HUD hidden against the server's own time, attribute, and metadata updates. */
    private fun onPacketOut(event: PlayerPacketOutEvent) {
        when (val packet = event.packet) {
            is SetTimePacket -> {
                if (containerOpen || packet.gameTime() < 0) return
                event.isCancelled = true
                player.sendPacket(SetTimePacket(canvas.hudHiddenWorldAge, packet.clocks()))
            }

            is EntityAttributesPacket -> {
                val capped = cappedMaxHealth() ?: return
                val properties = packet.properties()
                if (packet.entityId() != player.entityId) return
                if (properties.none { it.attribute() == Attribute.MAX_HEALTH && it != capped }) return
                event.isCancelled = true
                val clamped = properties.map { if (it.attribute() == Attribute.MAX_HEALTH) capped else it }
                player.sendPacket(EntityAttributesPacket(player.entityId, clamped))
            }

            is EntityMetaDataPacket -> {
                if (packet.entityId() != player.entityId) return
                val absorption = packet.entries()[ABSORPTION]?.value() ?: return
                if (absorption == 0f) return
                event.isCancelled = true
                player.sendPacket(EntityMetaDataPacket(player.entityId, packet.entries() + (ABSORPTION to ZERO)))
            }

            is OpenWindowPacket -> {
                containerOpen = true
                player.sendPacket(SetTimePacket(player.instance.worldAge, emptyMap()))
            }

            is CloseWindowPacket -> {
                player.scheduler().scheduleNextTick { if (open) containerClosed() }
            }

            else -> {
                Unit
            }
        }
    }

    private fun containerClosed() {
        if (!containerOpen) return
        containerOpen = false
        player.sendPacket(SetTimePacket(canvas.hudHiddenWorldAge, emptyMap()))
        // Closing a container resyncs the real hotbar after this tick's packets.
        player.scheduler().scheduleNextTick { if (open) blankHotbar() }
    }

    /** Ends the window after the player left it, restoring them unless they disconnected. */
    private fun end(restore: Boolean) {
        if (!stop()) return
        if (restore) restore()
        listener.onClose()
    }

    /** Stops listening; false when already stopped. */
    private fun stop(): Boolean {
        if (!open) return false
        open = false
        player.eventNode().removeChild(node)
        player.removeTag(ACTIVE)
        return true
    }

    private fun restore() {
        player.sendPacket(DestroyEntitiesPacket(listOf(display.entityId, clickTarget.entityId)))
        player.setGameMode(gameMode)
        player.isAllowFlying = allowFlying
        player.isFlying = flying
        player.flyingSpeed = flyingSpeed
        player.teleport(position)
        player.sendPacket(SetTimePacket(player.instance.worldAge, emptyMap()))
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

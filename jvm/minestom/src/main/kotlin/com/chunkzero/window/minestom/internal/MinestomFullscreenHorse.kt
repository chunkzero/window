package com.chunkzero.window.minestom.internal

import com.chunkzero.window.manifest.HorseEntry
import net.kyori.adventure.text.Component
import net.minestom.server.component.DataComponents
import net.minestom.server.coordinate.Pos
import net.minestom.server.coordinate.Vec
import net.minestom.server.entity.Entity
import net.minestom.server.entity.EntityType
import net.minestom.server.entity.EquipmentSlot
import net.minestom.server.entity.Player
import net.minestom.server.entity.attribute.Attribute
import net.minestom.server.entity.metadata.animal.AbstractHorseMeta
import net.minestom.server.item.ItemStack
import net.minestom.server.item.Material
import net.minestom.server.network.packet.server.play.ActionBarPacket
import net.minestom.server.network.packet.server.play.CameraPacket
import net.minestom.server.network.packet.server.play.DestroyEntitiesPacket
import net.minestom.server.network.packet.server.play.EntityAttributesPacket
import net.minestom.server.network.packet.server.play.EntityEquipmentPacket
import net.minestom.server.network.packet.server.play.SetPassengersPacket
import net.minestom.server.network.packet.server.play.SpawnEntityPacket

/**
 * The locked view of a horse-mode full-screen window, as [horse] describes.
 *
 * [player]'s camera spectates an invisible client-only bat at their eye, so their view stays still. They ride an
 * invisible, saddled, client-only horse wearing the pack's cursor armour. Every client tick, the client turns the horse
 * it steers to the player's yaw and half their pitch, and the pack's entity shader draws the cursor from the armour. As
 * a passenger, the client still sends the player's rotation every tick, which the server hit-tests as usual. The
 * client does not draw its own player, or its hotbar and hearts, while another entity is the camera.
 */
internal class MinestomFullscreenHorse(
    private val player: Player,
    private val horse: HorseEntry,
) {
    private val camera =
        Entity(EntityType.BAT).apply {
            isInvisible = true
            isSilent = true
            setNoGravity(true)
        }
    private val mount =
        Entity(EntityType.HORSE).apply {
            isInvisible = true
            isSilent = true
            setNoGravity(true)
            editEntityMeta(AbstractHorseMeta::class.java) { meta ->
                // The client only accepts a saddle on a tamed horse.
                meta.isTamed = true
                meta.isNoAi = true
            }
        }

    /** Spawns the bat at [eye] and the horse at its offset, both facing yaw and pitch 0, then locks the view. */
    fun lock(eye: Pos) {
        spawn(camera, eye.sub(0.0, camera.eyeHeight, 0.0))
        spawn(mount, eye.add(horse.offset[0], horse.offset[1], horse.offset[2]))
        val equipment = mapOf(EquipmentSlot.SADDLE to SADDLE, EquipmentSlot.BODY to armor(horse.armor))
        player.sendPacket(EntityEquipmentPacket(mount.entityId, equipment))
        // The client steers the horse, so it must not move it.
        val still =
            listOf(Attribute.MOVEMENT_SPEED, Attribute.JUMP_STRENGTH).map {
                EntityAttributesPacket.Property(it, 0.0, emptyList())
            }
        player.sendPacket(EntityAttributesPacket(mount.entityId, still))
        player.sendPacket(SetPassengersPacket(mount.entityId, listOf(player.entityId)))
        // Replaces the dismount hint the client shows on mounting.
        player.sendPacket(ActionBarPacket(Component.empty()))
        player.sendPacket(CameraPacket(camera.entityId))
    }

    /** Returns the camera to the player, dismounts them, and removes the bat and horse. */
    fun unlock() {
        player.sendPacket(CameraPacket(player.entityId))
        player.sendPacket(SetPassengersPacket(mount.entityId, emptyList()))
        player.sendPacket(DestroyEntitiesPacket(listOf(camera.entityId, mount.entityId)))
    }

    private fun spawn(
        entity: Entity,
        at: Pos,
    ) {
        player.sendPacket(SpawnEntityPacket(entity.entityId, entity.uuid, entity.entityType, at, 0f, 0, Vec.ZERO))
        player.sendPacket(entity.metadataPacket)
    }

    private companion object {
        /** Without an equipment asset the saddle draws nothing, but the horse still counts as saddled. */
        val SADDLE = equippable(Material.SADDLE, null)

        fun armor(asset: String) = equippable(Material.COPPER_HORSE_ARMOR, asset)

        fun equippable(
            material: Material,
            asset: String?,
        ): ItemStack {
            val equippable = requireNotNull(material.prototype().get(DataComponents.EQUIPPABLE))
            return ItemStack.of(material).with(DataComponents.EQUIPPABLE, equippable.withAssetId(asset))
        }
    }
}

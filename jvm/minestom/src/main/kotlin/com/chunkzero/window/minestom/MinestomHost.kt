package com.chunkzero.window.minestom

import com.chunkzero.window.host.ContainerKind
import com.chunkzero.window.host.ContainerListener
import com.chunkzero.window.host.HudDescriptor
import com.chunkzero.window.host.HudOutput
import com.chunkzero.window.host.OpenContainer
import com.chunkzero.window.host.WindowHost
import com.chunkzero.window.host.WindowItem
import com.chunkzero.window.minestom.internal.MinestomContainer
import com.chunkzero.window.minestom.internal.itemStack
import com.chunkzero.window.minestom.internal.showHud
import net.kyori.adventure.text.Component
import net.minestom.server.entity.Player
import net.minestom.server.item.ItemStack
import net.minestom.server.tag.Tag

/**
 * The Minestom [WindowHost] for [player].
 *
 * Containers listen on a child of the player's event node, and scheduled work runs on the player's scheduler, so
 * Window state for this player must be changed from the thread that ticks the player.
 */
public class MinestomHost private constructor(
    public val player: Player,
) : WindowHost<ItemStack> {
    override fun open(
        kind: ContainerKind,
        title: Component,
        listener: ContainerListener,
    ): OpenContainer<ItemStack> = MinestomContainer.open(player, kind, title, listener)

    override fun showHud(hud: HudDescriptor): HudOutput = showHud(player, hud)

    override fun item(item: WindowItem): ItemStack = itemStack(item)

    override fun scheduleNextTick(task: Runnable) {
        player.scheduler().scheduleNextTick(task)
    }

    public companion object {
        private val TAG = Tag.Transient<MinestomHost>("window:host")

        /** The host for [player], shared by every view and HUD shown to them. */
        @JvmStatic
        public fun of(player: Player): MinestomHost = player.updateAndGetTag(TAG) { it ?: MinestomHost(player) }
    }
}
